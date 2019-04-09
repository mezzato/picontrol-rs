use crate::picontrol;

pub const KB_RESET: u32 = request_code_none!(picontrol::KB_IOC_MAGIC, 12) as u32; //_IO(picontrol::KB_IOC_MAGIC, 12); // reset the piControl driver including the config file
pub const KB_GET_DEVICE_INFO_LIST: u32 = request_code_none!(picontrol::KB_IOC_MAGIC, 13) as u32; // get the device info of all detected devices
pub const KB_GET_DEVICE_INFO: u32 = request_code_none!(picontrol::KB_IOC_MAGIC, 14) as u32; // get the device info of one device
pub const KB_FIND_VARIABLE: u32 = request_code_none!(picontrol::KB_IOC_MAGIC, 17) as u32; // find a varible defined in piCtory
pub const KB_GET_VALUE: u32 = request_code_none!(picontrol::KB_IOC_MAGIC, 15) as u32; // get the value of one bit in the process image
pub const KB_SET_VALUE: u32 = request_code_none!(picontrol::KB_IOC_MAGIC, 16) as u32; // set the value of one bit in the process image

ioctl_none_bad!(reset, KB_RESET);
ioctl_read_bad!(
    getDeviceInfoList,
    KB_GET_DEVICE_INFO_LIST,
    picontrol::SDeviceInfo
);
ioctl_read_bad!(getVariableInfo, KB_FIND_VARIABLE, picontrol::SPIVariable);
ioctl_read_bad!(getBitValue, KB_GET_VALUE, picontrol::SPIValue);
ioctl_read_bad!(setBitValue, KB_SET_VALUE, picontrol::SPIValue);
