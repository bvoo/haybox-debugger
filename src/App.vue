<script setup lang="ts">
import { Button } from '@/components/ui/button';
import { invoke } from '@tauri-apps/api/core';
import { ref, onMounted, onUnmounted, computed } from 'vue';

// Type definitions for the data returned from Tauri commands
interface DeviceStatus {
  default_mode_connected: boolean;
  config_mode_connected: boolean;
  bootsel_mode_connected: boolean;
  switch_mode_connected: boolean;
  xinput_installed: boolean;
  gamecube_adapter_connected: boolean;
  winusb_installed: boolean;
}

interface DeviceInfo {
  default_mode: DeviceDetails;
  config_mode: DeviceDetails;
  bootsel_mode: DeviceDetails;
  switch_mode: DeviceDetails;
}

interface DeviceDetails {
  vid: number;
  pid: number;
  name: string;
}

interface DriverOperationResult {
  message: string;
  success: boolean;
}

interface DriverInfo {
  current_driver: string;
  available_drivers: string[];
}

// Device status references
const hayboxDefaultModeConnected = ref(false);
const hayboxConfigModeConnected = ref(false);
const hayboxBootselModeConnected = ref(false);
const hayboxSwitchModeConnected = ref(false);
const xinputInstalled = ref(false);
const gamecubeAdapterConnected = ref(false);
const winusbInstalled = ref(false);
const statusMessage = ref('');
const operationInProgress = ref(false);

// Centralized device information from backend
const deviceInfo = ref<DeviceInfo>({
  default_mode: { vid: 0, pid: 0, name: '' },
  config_mode: { vid: 0, pid: 0, name: '' },
  bootsel_mode: { vid: 0, pid: 0, name: '' },
  switch_mode: { vid: 0, pid: 0, name: '' },
});

// Add new refs for driver management
const currentDriver = ref('');
const availableDrivers = ref<string[]>([]);
const selectedDriver = ref('WinUSB');

// Add computed property for any mode connected
const anyModeConnected = computed(() => 
  hayboxDefaultModeConnected.value || 
  hayboxConfigModeConnected.value || 
  hayboxBootselModeConnected.value || 
  hayboxSwitchModeConnected.value
);

// Function to get device status directly
async function checkDeviceStatus(isManualRefresh = false) {
  try {
    if (isManualRefresh) {
      operationInProgress.value = true;
      statusMessage.value = 'Checking device status...';
    }
    
    const result = await invoke<DeviceStatus>('get_device_status');
    updateDeviceStatus(result);
    
    if (isManualRefresh) {
      statusMessage.value = 'Device status refreshed';
      setTimeout(() => statusMessage.value = '', 2000);
    }
  } catch (error) {
    console.error('Error checking devices:', error);
    if (isManualRefresh) {
      statusMessage.value = `Error checking devices: ${error}`;
    }
  } finally {
    if (isManualRefresh) operationInProgress.value = false;
  }
}

// Function to update device status from event or direct check
function updateDeviceStatus(status: DeviceStatus) {
  hayboxDefaultModeConnected.value = status.default_mode_connected;
  hayboxConfigModeConnected.value = status.config_mode_connected;
  hayboxBootselModeConnected.value = status.bootsel_mode_connected;
  hayboxSwitchModeConnected.value = status.switch_mode_connected;
  xinputInstalled.value = status.xinput_installed;
  gamecubeAdapterConnected.value = status.gamecube_adapter_connected;
  winusbInstalled.value = status.winusb_installed;
}

// Function to get device identifiers directly
async function getDeviceIdentifiers() {
  try {
    const result = await invoke<DeviceInfo>('get_device_identifiers');
    deviceInfo.value = result;
  } catch (error) {
    console.error('Error getting device identifiers:', error);
  }
}

// Function to get current driver info
async function getDriverInfo() {
  try {
    const result = await invoke<DriverInfo>('get_driver_info');
    currentDriver.value = result.current_driver;
    availableDrivers.value = result.available_drivers;
  } catch (error) {
    console.error('Error getting driver info:', error);
    statusMessage.value = `Error getting driver info: ${error}`;
  }
}

// Function to replace driver
async function replaceDriver() {
  if (!confirm(`Are you sure you want to replace the current driver with ${selectedDriver.value}?`)) {
    return;
  }
  
  try {
    operationInProgress.value = true;
    statusMessage.value = `Installing ${selectedDriver.value} driver...`;
    
    const result = await invoke<DriverOperationResult>('replace_driver', {
      driverName: selectedDriver.value
    });
    statusMessage.value = result.message;
    await checkDeviceStatus();
    await getDriverInfo();
  } catch (error) {
    statusMessage.value = `Error installing driver: ${error}`;
  } finally {
    operationInProgress.value = false;
  }
}

// Function to uninstall XInput driver
async function uninstallXInput() {
  if (!confirm('Are you sure you want to uninstall the XInput driver?')) {
    return;
  }
  
  try {
    operationInProgress.value = true;
    statusMessage.value = 'Uninstalling XInput driver...';
    
    const result = await invoke<DriverOperationResult>('uninstall_xinput');
    statusMessage.value = result.message;
    await checkDeviceStatus();
  } catch (error) {
    statusMessage.value = `Error uninstalling XInput driver: ${error}`;
  } finally {
    operationInProgress.value = false;
  }
}

// Function to reinstall XInput driver
async function reinstallXInput() {
  try {
    operationInProgress.value = true;
    statusMessage.value = 'Reinstalling XInput driver...';
    
    const result = await invoke<DriverOperationResult>('reinstall_xinput');
    statusMessage.value = result.message;
    await checkDeviceStatus();
  } catch (error) {
    statusMessage.value = `Error reinstalling XInput driver: ${error}`;
  } finally {
    operationInProgress.value = false;
  }
}

// Function to install WinUSB driver
async function installWinUSBDriver() {
  if (!confirm('Are you sure you want to install the WinUSB driver for your GameCube adapter?')) {
    return;
  }
  
  try {
    operationInProgress.value = true;
    statusMessage.value = 'Installing WinUSB driver...';
    
    const result = await invoke<DriverOperationResult>('install_winusb');
    statusMessage.value = result.message;
    await checkDeviceStatus();
  } catch (error) {
    statusMessage.value = `Error installing WinUSB driver: ${error}`;
  } finally {
    operationInProgress.value = false;
  }
}

// Initialize data and setup event listeners on mount
let pollInterval: number;

onMounted(async () => {
  // Get initial device identifiers and status
  await getDeviceIdentifiers();
  await checkDeviceStatus();
  await getDriverInfo();
  
  // Set up polling every second
  pollInterval = setInterval(() => checkDeviceStatus(false), 500);
});

// Cleanup event listeners on unmount
onUnmounted(() => {
  // Clean up polling interval
  if (pollInterval) {
    clearInterval(pollInterval);
  }
});
</script>

<template>
  <main class="flex flex-col p-8 max-w-2xl mx-auto">
    <h1 class="text-3xl font-bold mb-6 text-center text-foreground">Haybox Debugger</h1>
    
    <div class="p-6 rounded-lg shadow mb-6 bg-card text-card-foreground border border-border">
      <h2 class="text-xl font-semibold mb-4">Device Status</h2>
      
      <div class="grid grid-cols-2 gap-4 mb-4">
        <div class="flex items-center">
          <div class="w-3 h-3 rounded-full mr-2" 
               :class="hayboxDefaultModeConnected ? 'bg-[#8024cc]' : 'bg-destructive'"></div>
          <div>
            <span>Default Mode</span>
            <div class="text-xs text-muted-foreground font-mono">
              VID: 0x2E8A, PID: 0x0004
            </div>
          </div>
        </div>
        <div class="flex justify-end items-center">
          <span class="px-3 py-1 rounded-full text-sm font-semibold shadow-sm" 
                :class="hayboxDefaultModeConnected ? 'bg-[#8024cc]/10 text-[#8024cc] border border-[#8024cc]/20' : 'bg-destructive/10 text-red-500/80 border border-destructive/80'">
            {{ hayboxDefaultModeConnected ? 'Connected' : 'Not Connected' }}
          </span>
        </div>

        <div class="flex items-center">
          <div class="w-3 h-3 rounded-full mr-2" 
               :class="hayboxConfigModeConnected ? 'bg-[#8024cc]' : 'bg-destructive'"></div>
          <div>
            <span>Config Mode</span>
            <div class="text-xs text-muted-foreground font-mono">
              VID: 0x2E8A, PID: 0x000A
            </div>
          </div>
        </div>
        <div class="flex justify-end items-center">
          <span class="px-3 py-1 rounded-full text-sm font-semibold shadow-sm" 
                :class="hayboxConfigModeConnected ? 'bg-[#8024cc]/10 text-[#8024cc] border border-[#8024cc]/20' : 'bg-destructive/10 text-red-500/80 border border-destructive/80'">
            {{ hayboxConfigModeConnected ? 'Connected' : 'Not Connected' }}
          </span>
        </div>

        <div class="flex items-center">
          <div class="w-3 h-3 rounded-full mr-2" 
               :class="hayboxBootselModeConnected ? 'bg-[#8024cc]' : 'bg-destructive'"></div>
          <div>
            <span>BOOTSEL Mode</span>
            <div class="text-xs text-muted-foreground font-mono">
              VID: 0x2E8A, PID: 0x0003
            </div>
          </div>
        </div>
        <div class="flex justify-end items-center">
          <span class="px-3 py-1 rounded-full text-sm font-semibold shadow-sm" 
                :class="hayboxBootselModeConnected ? 'bg-[#8024cc]/10 text-[#8024cc] border border-[#8024cc]/20' : 'bg-destructive/10 text-red-500/80 border border-destructive/80'">
            {{ hayboxBootselModeConnected ? 'Connected' : 'Not Connected' }}
          </span>
        </div>
      </div>
    </div>

    <div class="p-6 rounded-lg shadow mb-6 bg-card text-card-foreground border border-border">
      <h2 class="text-xl font-semibold mb-4">GameCube Adapter Status</h2>
      
      <div class="grid grid-cols-2 gap-4 mb-4">
        <div class="flex items-center">
          <div class="w-3 h-3 rounded-full mr-2" 
               :class="gamecubeAdapterConnected ? 'bg-[#8024cc]' : 'bg-destructive'"></div>
          <div>
            <span>GameCube Adapter</span>
            <div class="text-xs text-muted-foreground font-mono">
              VID: 0x057E, PID: 0x0337
            </div>
          </div>
        </div>
        <div class="flex justify-end items-center">
          <span class="px-3 py-1 rounded-full text-sm font-semibold shadow-sm" 
                :class="gamecubeAdapterConnected ? 'bg-[#8024cc]/10 text-[#8024cc] border border-[#8024cc]/20' : 'bg-destructive/10 text-red-500/80 border border-destructive/80'">
            {{ gamecubeAdapterConnected ? 'Connected' : 'Not Connected' }}
          </span>
        </div>

        <div class="flex items-center">
          <div class="w-3 h-3 rounded-full mr-2" 
               :class="winusbInstalled ? 'bg-[#8024cc]' : 'bg-destructive'"></div>
          <span>WinUSB Driver</span>
        </div>
        <div class="flex justify-end items-center">
          <span class="px-3 py-1 rounded-full text-sm font-semibold shadow-sm" 
                :class="winusbInstalled ? 'bg-[#8024cc]/10 text-[#8024cc] border border-[#8024cc]/20' : 'bg-destructive/10 text-red-500/80 border border-destructive/80'">
            {{ winusbInstalled ? 'Installed' : 'Not Installed' }}
          </span>
        </div>
      </div>

      <div class="mt-6">
        <p class="mb-4 text-sm text-muted-foreground">
          Install WinUSB driver for your GameCube adapter to use it with Dolphin emulator.
          This operation requires administrator privileges.
        </p>
        
        <Button @click="installWinUSBDriver" variant="default" 
                :disabled="operationInProgress || !gamecubeAdapterConnected || winusbInstalled">
          {{ winusbInstalled ? 'WinUSB Driver Already Installed' : 'Install WinUSB Driver' }}
        </Button>
      </div>
    </div>
    
    <div v-if="statusMessage" class="mt-4 p-4 rounded-lg" 
         :class="statusMessage.includes('Error') ? 'bg-destructive/20 text-red-500/80' : 'bg-primary/20 text-primary-foreground'">
      {{ statusMessage }}
    </div>
  </main>
</template>

<style>
body {
  margin: 0;
  padding: 0;
  font-family: system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 
    'Helvetica Neue', Arial, sans-serif;
  background-color: var(--color-background);
  color: var(--color-foreground);
}
</style>
