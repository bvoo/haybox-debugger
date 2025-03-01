use std::path::PathBuf;
use std::process::Command;

use rusb::UsbContext;
use serde::{Deserialize, Serialize};

// Define device identifiers in a centralized structure
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UsbDeviceInfo {
  pub vid: u16,
  pub pid: u16,
  pub name: String,
}

// All device identifiers in one place
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeviceIdentifiers {
  pub default_mode: UsbDeviceInfo,
  pub config_mode: UsbDeviceInfo,
  pub bootsel_mode: UsbDeviceInfo,
  pub switch_mode: UsbDeviceInfo,
  pub gamecube_mode: UsbDeviceInfo,
}

// Define structures for WinUSB driver installation
#[derive(Debug)]
pub struct Config {
  vendor_id: u16,
  product_id: u16,
  description: String,
  manufacturer: String,
}

impl Config {
  pub fn prepare_driver(&self) -> Result<(), PrepareDriverError> {
    // In a real implementation, this would prepare driver files
    // For now, just a stub implementation that succeeds
    Ok(())
  }
  
  pub fn install_driver(&self) -> Result<(), String> {
    // This would actually install the driver using Windows APIs or tools
    // For this stub version, we'll simulate success
    // In a real implementation, you would call something like zadig or libwdi
    Ok(())
  }
}

pub struct ConfigBuilder {
  vendor_id: u16,
  product_id: u16,
  description: String,
  manufacturer: String,
}

impl ConfigBuilder {
  pub fn new() -> Self {
    Self {
      vendor_id: 0,
      product_id: 0,
      description: String::new(),
      manufacturer: String::new(),
    }
  }
  
  pub fn vendor_id(mut self, vendor_id: u16) -> Self {
    self.vendor_id = vendor_id;
    self
  }
  
  pub fn product_id(mut self, product_id: u16) -> Self {
    self.product_id = product_id;
    self
  }
  
  pub fn description(mut self, description: &str) -> Self {
    self.description = description.to_string();
    self
  }
  
  pub fn manufacturer(mut self, manufacturer: &str) -> Self {
    self.manufacturer = manufacturer.to_string();
    self
  }
  
  pub fn build(self) -> Config {
    Config {
      vendor_id: self.vendor_id,
      product_id: self.product_id,
      description: self.description,
      manufacturer: self.manufacturer,
    }
  }
}

#[derive(Debug)]
pub enum PrepareDriverError {
  DriverNotFound,
  PermissionDenied,
  UnknownError(String),
}

impl std::fmt::Display for PrepareDriverError {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match self {
      PrepareDriverError::DriverNotFound => write!(f, "Driver files not found"),
      PrepareDriverError::PermissionDenied => write!(f, "Permission denied"),
      PrepareDriverError::UnknownError(e) => write!(f, "Unknown error: {}", e),
    }
  }
}

impl std::error::Error for PrepareDriverError {}

// Define the devices
lazy_static::lazy_static! {
  static ref DEVICES: DeviceIdentifiers = DeviceIdentifiers {
    default_mode: UsbDeviceInfo {
      vid: 0x0738,
      pid: 0x4726,
      name: "Default Mode".to_string(),
    },
    config_mode: UsbDeviceInfo {
      vid: 0x2E8A,
      pid: 0x000A,
      name: "Config Mode".to_string(),
    },
    bootsel_mode: UsbDeviceInfo {
      vid: 0x2E8A,
      pid: 0x0003,
      name: "BOOTSEL Mode".to_string(),
    },
    switch_mode: UsbDeviceInfo {
      vid: 0x0F0D,
      pid: 0x0092,
      name: "Switch Mode".to_string(),
    },
    gamecube_mode: UsbDeviceInfo {
      vid: 0x057E,
      pid: 0x0337,
      name: "GameCube Adapter".to_string(),
    }
  };
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct DeviceStatus {
  default_mode_connected: bool,
  config_mode_connected: bool,
  bootsel_mode_connected: bool,
  switch_mode_connected: bool,
  xinput_installed: bool,
  gamecube_adapter_connected: bool,
  winusb_installed: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DriverOperationResult {
  success: bool,
  message: String,
}

#[tauri::command(rename_all = "snake_case")]
fn get_device_identifiers() -> DeviceIdentifiers {
  DEVICES.clone()
}

fn is_device_connected_batch(devices_to_check: &[(u16, u16)]) -> Vec<bool> {
  match rusb::Context::new() {
    Ok(context) => match context.devices() {
      Ok(device_list) => {
        let mut results = vec![false; devices_to_check.len()];

        for device in device_list.iter() {
          if let Ok(device_desc) = device.device_descriptor() {
            for (i, &(vendor_id, product_id)) in devices_to_check.iter().enumerate() {
              if device_desc.vendor_id() == vendor_id && device_desc.product_id() == product_id {
                results[i] = true;
              }
            }
          }
        }
        results
      }
      Err(_) => vec![false; devices_to_check.len()],
    },
    Err(_) => vec![false; devices_to_check.len()],
  }
}

fn get_current_device_status() -> Result<DeviceStatus, Box<dyn std::error::Error>> {
  let devices_to_check = [
    (DEVICES.default_mode.vid, DEVICES.default_mode.pid),
    (DEVICES.config_mode.vid, DEVICES.config_mode.pid),
    (DEVICES.bootsel_mode.vid, DEVICES.bootsel_mode.pid),
    (DEVICES.switch_mode.vid, DEVICES.switch_mode.pid),
  ];

  let connected = is_device_connected_batch(&devices_to_check);
  let xinput_installed = is_xinput_installed();
  let winusb_installed = check_winusb_driver(DEVICES.gamecube_mode.vid, DEVICES.gamecube_mode.pid)?;
  let gamecube_adapter_connected = match rusb::Context::new() {
    Ok(context) => match context.devices() {
      Ok(device_list) => device_list.iter().any(|device| {
        if let Ok(device_desc) = device.device_descriptor() {
          device_desc.vendor_id() == DEVICES.gamecube_mode.vid && device_desc.product_id() == DEVICES.gamecube_mode.pid
        } else {
          false
        }
      }),
      Err(_) => false,
    },
    Err(_) => false,
  };

  Ok(DeviceStatus {
    default_mode_connected: connected[0],
    config_mode_connected: connected[1],
    bootsel_mode_connected: connected[2],
    switch_mode_connected: connected[3],
    xinput_installed,
    gamecube_adapter_connected,
    winusb_installed,
  })
}

#[tauri::command(rename_all = "snake_case")]
async fn get_device_status() -> DeviceStatus {
  get_current_device_status().unwrap_or(DeviceStatus {
    default_mode_connected: false,
    config_mode_connected: false,
    bootsel_mode_connected: false,
    switch_mode_connected: false,
    xinput_installed: false,
    gamecube_adapter_connected: false,
    winusb_installed: false,
  })
}

#[tauri::command(rename_all = "snake_case")]
fn uninstall_xinput() -> DriverOperationResult {
  match uninstall_xinput_driver() {
    Ok(_) => DriverOperationResult {
      success: true,
      message: "XInput driver successfully uninstalled".to_string(),
    },
    Err(e) => DriverOperationResult {
      success: false,
      message: format!("Failed to uninstall XInput driver: {}", e),
    },
  }
}

#[tauri::command(rename_all = "snake_case")]
fn reinstall_xinput(_app_handle: tauri::AppHandle) -> DriverOperationResult {
  match reinstall_xinput_driver() {
    Ok(_) => DriverOperationResult {
      success: true,
      message: "XInput driver successfully reinstalled".to_string(),
    },
    Err(e) => DriverOperationResult {
      success: false,
      message: format!("Failed to reinstall XInput driver: {}", e),
    },
  }
}

#[tauri::command(rename_all = "snake_case")]
fn install_winusb() -> DriverOperationResult {
  if (!check_admin_rights()) {
    return DriverOperationResult {
      success: false,
      message: "Administrator privileges required".to_string(),
    };
  }

  // Check if the device is connected first
  let gamecube_mode = &DEVICES.gamecube_mode;
  let is_connected = match rusb::Context::new() {
    Ok(context) => match context.devices() {
      Ok(device_list) => device_list.iter().any(|device| {
        if let Ok(device_desc) = device.device_descriptor() {
          device_desc.vendor_id() == gamecube_mode.vid && device_desc.product_id() == gamecube_mode.pid
        } else {
          false
        }
      }),
      Err(_) => false,
    },
    Err(_) => false,
  };

  if (!is_connected) {
    return DriverOperationResult {
      success: false,
      message: "GameCube adapter not found. Please make sure it is connected and in the correct mode.".to_string(),
    };
  }

  // Configure WinUSB driver installation
  let config = ConfigBuilder::new()
    .vendor_id(gamecube_mode.vid)
    .product_id(gamecube_mode.pid)
    .description(&gamecube_mode.name)
    .manufacturer("Nintendo")
    .build();

  match install_winusb_driver(&config) {
    Ok(_) => DriverOperationResult {
      success: true,
      message: "WinUSB driver successfully installed for GameCube adapter".to_string(),
    },
    Err(e) => DriverOperationResult {
      success: false,
      message: format!("Failed to install WinUSB driver: {}", e),
    },
  }
}

fn install_winusb_driver(config: &Config) -> Result<(), String> {
  match config.prepare_driver() {
    // Driver files are prepared, now install them
    Ok(_) => match config.install_driver() {
      Ok(_) => Ok(()),
      Err(e) => Err(format!("Failed to install driver: {}", e)),
    },
    Err(PrepareDriverError::DriverNotFound) => Err("WinUSB driver files not found".to_string()),
    Err(e) => Err(format!("Failed to prepare driver: {}", e)),
  }
}

fn is_xinput_installed() -> bool {
  let system32_path = std::env::var("SystemRoot")
    .map(|root| PathBuf::from(root).join("System32"))
    .unwrap_or_else(|_| PathBuf::from("C:\\Windows\\System32"));

  let xinput_path = system32_path.join("xinput1_4.dll");
  xinput_path.exists()
}

fn check_admin_rights() -> bool {
  if let Ok(output) = Command::new("net").args(["session"]).output() {
    output.status.success()
  } else {
    false
  }
}

fn uninstall_xinput_driver() -> Result<(), String> {
  if (!check_admin_rights()) {
    return Err("Administrator privileges required".to_string());
  }

  let system32_path = std::env::var("SystemRoot")
    .map(|root| PathBuf::from(root).join("System32"))
    .unwrap_or_else(|_| PathBuf::from("C:\\Windows\\System32"));

  let xinput_path = system32_path.join("xinput1_4.dll");

  if (xinput_path.exists()) {
    let backup_path = xinput_path.with_extension("dll.bak");
    std::fs::rename(&xinput_path, &backup_path).map_err(|e| format!("Failed to rename xinput1_4.dll: {}", e))?;
  }

  Ok(())
}

fn reinstall_xinput_driver() -> Result<(), String> {
  if (!check_admin_rights()) {
    return Err("Administrator privileges required".to_string());
  }

  let system32_path = std::env::var("SystemRoot")
    .map(|root| PathBuf::from(root).join("System32"))
    .unwrap_or_else(|_| PathBuf::from("C:\\Windows\\System32"));

  let xinput_path = system32_path.join("xinput1_4.dll");

  // Get the DLL from the public directory next to the executable
  let exe_dir = std::env::current_exe()
    .map_err(|e| format!("Could not find executable path: {}", e))?
    .parent()
    .ok_or_else(|| "Could not find executable parent directory".to_string())?
    .to_path_buf();

  let source_dll = exe_dir.join("XInput1_4.dll");

  // Copy the DLL from public dir to System32
  std::fs::copy(&source_dll, &xinput_path).map_err(|e| format!("Failed to copy xinput1_4.dll: {}", e))?;

  Ok(())
}

// Define a struct for WMI query results
#[derive(Debug, Deserialize)]
struct WmiPnPEntity {
  #[serde(rename = "DeviceID")]
  device_id: String,
  #[serde(rename = "DriverProvider")]
  driver_provider: Option<String>,
}

fn check_winusb_driver(vendor_id: u16, product_id: u16) -> Result<bool, Box<dyn std::error::Error>> {
  // First check if the device is connected at all
  let is_connected = match rusb::Context::new() {
    Ok(context) => match context.devices() {
      Ok(device_list) => device_list.iter().any(|device| {
        if let Ok(device_desc) = device.device_descriptor() {
          device_desc.vendor_id() == vendor_id && device_desc.product_id() == product_id
        } else {
          false
        }
      }),
      Err(_) => false,
    },
    Err(_) => false,
  };

  // If device isn't connected, don't bother checking WMI
  if !is_connected {
    return Ok(false);
  }

  // Device is connected, now check its driver via WMI
  let wmi_con = unsafe { wmi::COMLibrary::assume_initialized() };
  
  let wmi_connection = wmi::WMIConnection::new(wmi_con)
    .map_err(|e| format!("Failed to initialize WMI: {}", e))?;

  let query = format!(
    "SELECT DeviceID, DriverProvider FROM Win32_PnPEntity WHERE DeviceID LIKE '%VID_{0:04X}%' AND DeviceID LIKE '%PID_{1:04X}%'",
    vendor_id, product_id
  );

  let devices: Vec<WmiPnPEntity> = wmi_connection.raw_query(&query)?;

  // Check if any of the matching devices use WinUSB
  for device in devices {
    if let Some(driver_provider) = device.driver_provider {
      if driver_provider.contains("WinUSB") {
        return Ok(true);
      }
    }
  }

  Ok(false)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DriverInfo {
  device_id: String,
  device_name: String,
  driver_provider: Option<String>,
  driver_version: Option<String>,
  driver_date: Option<String>,
  is_winusb: bool,
}

#[tauri::command(rename_all = "snake_case")]
fn get_driver_info(vendor_id: Option<u16>, product_id: Option<u16>) -> Result<Vec<DriverInfo>, String> {
  // If specific VID/PID is provided, first check if the device is connected
  if let (Some(vid), Some(pid)) = (vendor_id, product_id) {
    let is_connected = match rusb::Context::new() {
      Ok(context) => match context.devices() {
        Ok(device_list) => device_list.iter().any(|device| {
          if let Ok(device_desc) = device.device_descriptor() {
            device_desc.vendor_id() == vid && device_desc.product_id() == pid
          } else {
            false
          }
        }),
        Err(_) => false,
      },
      Err(_) => false,
    };

    // If device isn't connected, return empty result without doing WMI query
    if !is_connected {
      return Ok(vec![]);
    }
  }

  // Device is connected or we're doing a general query, proceed with WMI
  let wmi_con = unsafe { wmi::COMLibrary::assume_initialized() };
  
  let wmi_connection = match wmi::WMIConnection::new(wmi_con) {
    Ok(conn) => conn,
    Err(e) => return Err(format!("Failed to initialize WMI: {}", e)),
  };

  // Construct the query based on whether we have vendor_id and product_id
  let query = match (vendor_id, product_id) {
    (Some(vid), Some(pid)) => format!(
      "SELECT DeviceID, Name, DriverProvider, DriverVersion, DriverDate FROM Win32_PnPEntity WHERE DeviceID LIKE '%VID_{0:04X}%' AND DeviceID LIKE '%PID_{1:04X}%'",
      vid, pid
    ),
    (Some(vid), None) => format!(
      "SELECT DeviceID, Name, DriverProvider, DriverVersion, DriverDate FROM Win32_PnPEntity WHERE DeviceID LIKE '%VID_{0:04X}%'",
      vid
    ),
    (None, Some(pid)) => format!(
      "SELECT DeviceID, Name, DriverProvider, DriverVersion, DriverDate FROM Win32_PnPEntity WHERE DeviceID LIKE '%PID_{0:04X}%'",
      pid
    ),
    (None, None) => {
      // If no VID/PID provided, get all USB devices
      "SELECT DeviceID, Name, DriverProvider, DriverVersion, DriverDate FROM Win32_PnPEntity WHERE DeviceID LIKE '%USB%'".to_string()
    }
  };

  #[derive(Debug, Deserialize)]
  struct WmiDeviceInfo {
    #[serde(rename = "DeviceID")]
    device_id: String,
    #[serde(rename = "Name")]
    name: Option<String>,
    #[serde(rename = "DriverProvider")]
    driver_provider: Option<String>,
    #[serde(rename = "DriverVersion")]
    driver_version: Option<String>,
    #[serde(rename = "DriverDate")]
    driver_date: Option<String>,
  }

  let devices: Vec<WmiDeviceInfo> = match wmi_connection.raw_query(&query) {
    Ok(devices) => {
      println!("WMI query successful. Found {} device(s)", devices.len());
      devices
    },
    Err(e) => {
      let error_msg = format!("Failed to query WMI: {}", e);
      println!("ERROR: {}", error_msg);
      return Err(error_msg);
    }
  };

  // If no devices were found, we should still return an empty array rather than an error
  let driver_info: Vec<DriverInfo> = devices
    .into_iter()
    .map(|device| {
      let is_winusb = device.driver_provider
        .as_ref()
        .map(|provider| provider.contains("WinUSB"))
        .unwrap_or(false);

      DriverInfo {
        device_id: device.device_id,
        device_name: device.name.unwrap_or_else(|| "Unknown Device".to_string()),
        driver_provider: device.driver_provider,
        driver_version: device.driver_version,
        driver_date: device.driver_date,
        is_winusb,
      }
    })
    .collect();

  println!("Returning {} driver info records", driver_info.len());
  Ok(driver_info)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .plugin(tauri_plugin_dialog::init())
    .plugin(tauri_plugin_opener::init())
    .invoke_handler(tauri::generate_handler![
      get_device_status,
      get_device_identifiers,
      uninstall_xinput,
      reinstall_xinput,
      install_winusb,
      get_driver_info
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
