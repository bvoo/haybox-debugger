<script setup lang="ts">
import { Button } from '@/components/ui/button';
import { invoke } from '@tauri-apps/api/core';
import { ref, onMounted, onUnmounted, computed } from 'vue';

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

const hayboxDefaultModeConnected = ref(false);
const hayboxConfigModeConnected = ref(false);
const hayboxBootselModeConnected = ref(false);
const hayboxSwitchModeConnected = ref(false);
const xinputInstalled = ref(false);
const gamecubeAdapterConnected = ref(false);
const winusbInstalled = ref(false);
const statusMessage = ref('');
const operationInProgress = ref(false);

const deviceInfo = ref<DeviceInfo>({
  default_mode: { vid: 0, pid: 0, name: '' },
  config_mode: { vid: 0, pid: 0, name: '' },
  bootsel_mode: { vid: 0, pid: 0, name: '' },
  switch_mode: { vid: 0, pid: 0, name: '' },
});

const currentDriver = ref('');
const availableDrivers = ref<string[]>([]);

const anyModeConnected = computed(
  () =>
    hayboxDefaultModeConnected.value ||
    hayboxConfigModeConnected.value ||
    hayboxBootselModeConnected.value ||
    hayboxSwitchModeConnected.value,
);

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
      setTimeout(() => (statusMessage.value = ''), 2000);
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

function updateDeviceStatus(status: DeviceStatus) {
  hayboxDefaultModeConnected.value = status.default_mode_connected;
  hayboxConfigModeConnected.value = status.config_mode_connected;
  hayboxBootselModeConnected.value = status.bootsel_mode_connected;
  hayboxSwitchModeConnected.value = status.switch_mode_connected;
  xinputInstalled.value = status.xinput_installed;
  gamecubeAdapterConnected.value = status.gamecube_adapter_connected;
  winusbInstalled.value = status.winusb_installed;
}

async function getDeviceIdentifiers() {
  try {
    const result = await invoke<DeviceInfo>('get_device_identifiers');
    deviceInfo.value = result;
  } catch (error) {
    console.error('Error getting device identifiers:', error);
  }
}

async function getDriverInfo() {
  try {
    const result = await invoke<DriverInfo>('get_driver_info');
    currentDriver.value = result.current_driver;
    availableDrivers.value = result.available_drivers;
  } catch (error) {
    console.error('Error getting driver info:', error);
  }
}

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

let pollInterval: number;

onMounted(async () => {
  await getDeviceIdentifiers();
  await checkDeviceStatus();
  await getDriverInfo();

  pollInterval = setInterval(() => checkDeviceStatus(false), 500);
});

onUnmounted(() => {
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
            <div class="text-xs text-muted-foreground font-mono">VID: 0x2E8A, PID: 0x0004</div>
          </div>
        </div>
        <div class="flex justify-end items-center">
          <span class="px-3 py-1 rounded-full text-sm font-semibold shadow-sm" :class="hayboxDefaultModeConnected
              ? 'bg-[#8024cc]/10 text-[#8024cc] border border-[#8024cc]/20'
              : 'bg-destructive/10 text-red-500/80 border border-destructive/80'
            ">
            {{ hayboxDefaultModeConnected ? 'Connected' : 'Not Connected' }}
          </span>
        </div>

        <div class="flex items-center">
          <div class="w-3 h-3 rounded-full mr-2" :class="hayboxConfigModeConnected ? 'bg-[#8024cc]' : 'bg-destructive'">
          </div>
          <div>
            <span>Config Mode</span>
            <div class="text-xs text-muted-foreground font-mono">VID: 0x2E8A, PID: 0x000A</div>
          </div>
        </div>
        <div class="flex justify-end items-center">
          <span class="px-3 py-1 rounded-full text-sm font-semibold shadow-sm" :class="hayboxConfigModeConnected
              ? 'bg-[#8024cc]/10 text-[#8024cc] border border-[#8024cc]/20'
              : 'bg-destructive/10 text-red-500/80 border border-destructive/80'
            ">
            {{ hayboxConfigModeConnected ? 'Connected' : 'Not Connected' }}
          </span>
        </div>

        <div class="flex items-center">
          <div class="w-3 h-3 rounded-full mr-2"
            :class="hayboxBootselModeConnected ? 'bg-[#8024cc]' : 'bg-destructive'"></div>
          <div>
            <span>BOOTSEL Mode</span>
            <div class="text-xs text-muted-foreground font-mono">VID: 0x2E8A, PID: 0x0003</div>
          </div>
        </div>
        <div class="flex justify-end items-center">
          <span class="px-3 py-1 rounded-full text-sm font-semibold shadow-sm" :class="hayboxBootselModeConnected
              ? 'bg-[#8024cc]/10 text-[#8024cc] border border-[#8024cc]/20'
              : 'bg-destructive/10 text-red-500/80 border border-destructive/80'
            ">
            {{ hayboxBootselModeConnected ? 'Connected' : 'Not Connected' }}
          </span>
        </div>

        <div class="flex items-center">
          <div class="w-3 h-3 rounded-full mr-2" :class="hayboxSwitchModeConnected ? 'bg-[#8024cc]' : 'bg-destructive'">
          </div>
          <div>
            <span>Switch Mode</span>
            <div class="text-xs text-muted-foreground font-mono">VID: 0x2E8A, PID: 0x0005</div>
          </div>
        </div>
        <div class="flex justify-end items-center">
          <span class="px-3 py-1 rounded-full text-sm font-semibold shadow-sm" :class="hayboxSwitchModeConnected
              ? 'bg-[#8024cc]/10 text-[#8024cc] border border-[#8024cc]/20'
              : 'bg-destructive/10 text-red-500/80 border border-destructive/80'
            ">
            {{ hayboxSwitchModeConnected ? 'Connected' : 'Not Connected' }}
          </span>
        </div>
      </div>
    </div>

    <div class="p-6 rounded-lg shadow mb-6 bg-card text-card-foreground border border-border">
      <h2 class="text-xl font-semibold mb-4">GameCube Adapter Status</h2>

      <div class="grid grid-cols-2 gap-4 mb-4">
        <div class="flex items-center">
          <div class="w-3 h-3 rounded-full mr-2" :class="gamecubeAdapterConnected ? 'bg-[#8024cc]' : 'bg-destructive'">
          </div>
          <div>
            <span>GameCube Adapter</span>
            <div class="text-xs text-muted-foreground font-mono">VID: 0x057E, PID: 0x0337</div>
          </div>
        </div>
        <div class="flex justify-end items-center">
          <span class="px-3 py-1 rounded-full text-sm font-semibold shadow-sm" :class="gamecubeAdapterConnected
              ? 'bg-[#8024cc]/10 text-[#8024cc] border border-[#8024cc]/20'
              : 'bg-destructive/10 text-red-500/80 border border-destructive/80'
            ">
            {{ gamecubeAdapterConnected ? 'Connected' : 'Not Connected' }}
          </span>
        </div>

        <div class="flex items-center">
          <div class="w-3 h-3 rounded-full mr-2" :class="winusbInstalled ? 'bg-[#8024cc]' : 'bg-destructive'"></div>
          <span>WinUSB Driver</span>
        </div>
        <div class="flex justify-end items-center">
          <span class="px-3 py-1 rounded-full text-sm font-semibold shadow-sm" :class="winusbInstalled
              ? 'bg-[#8024cc]/10 text-[#8024cc] border border-[#8024cc]/20'
              : 'bg-destructive/10 text-red-500/80 border border-destructive/80'
            ">
            {{ winusbInstalled ? 'Installed' : 'Not Installed' }}
          </span>
        </div>
      </div>

      <div class="mt-6">
        <p class="mb-4 text-sm text-muted-foreground">
          Install the WinUSB driver for your GameCube adapter to use it with Dolphin. This operation requires
          administrator privileges.
        </p>

        <Button @click="installWinUSBDriver" variant="default"
          :disabled="operationInProgress || !gamecubeAdapterConnected || winusbInstalled">
          {{ winusbInstalled ? 'WinUSB Driver Already Installed' : 'Install WinUSB Driver' }}
        </Button>
      </div>
    </div>

    <div class="p-6 rounded-lg shadow mb-6 bg-card text-card-foreground border border-border">
      <h2 class="text-xl font-semibold mb-4">XInput Driver Status</h2>

      <div class="grid grid-cols-2 gap-4 mb-4">
        <div class="flex items-center">
          <div class="w-3 h-3 rounded-full mr-2" :class="xinputInstalled ? 'bg-[#8024cc]' : 'bg-destructive'"></div>
          <span>XInput Driver</span>
        </div>
        <div class="flex justify-end items-center">
          <span class="px-3 py-1 rounded-full text-sm font-semibold shadow-sm" :class="xinputInstalled
              ? 'bg-[#8024cc]/10 text-[#8024cc] border border-[#8024cc]/20'
              : 'bg-destructive/10 text-red-500/80 border border-destructive/80'
            ">
            {{ xinputInstalled ? 'Installed' : 'Not Installed' }}
          </span>
        </div>
      </div>

      <div class="mt-6">
        <p class="mb-4 text-sm text-muted-foreground">
          Manage the XInput driver for your Haybox controller. Uninstalling may help resolve conflicts, while
          reinstalling can fix driver issues. These operations require administrator privileges.
        </p>

        <div class="flex gap-4">
          <Button @click="uninstallXInput" variant="destructive"
            :disabled="operationInProgress || !xinputInstalled || !anyModeConnected">
            Uninstall XInput Driver
          </Button>
          <Button @click="reinstallXInput" variant="ghost"
            :disabled="operationInProgress || (!xinputInstalled && !anyModeConnected)">
            {{ xinputInstalled ? 'Reinstall XInput Driver' : 'Install XInput Driver' }}
          </Button>
        </div>
      </div>
    </div>
    <div v-if="statusMessage" class="mt-4 p-4 rounded-lg" :class="statusMessage.includes('Error') ? 'bg-destructive/20 text-red-500/80' : 'bg-primary/20 text-primary-foreground'
      ">
      {{ statusMessage }}
    </div>
  </main>
</template>

<style>
body {
  margin: 0;
  padding: 0;
  font-family:
    system-ui,
    -apple-system,
    BlinkMacSystemFont,
    'Segoe UI',
    Roboto,
    'Helvetica Neue',
    Arial,
    sans-serif;
  background-color: var(--color-background);
  color: var(--color-foreground);
}
</style>
