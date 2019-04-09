//! # picontrol
//!
//! A library to control the Revolution Pi industrial PLC based on the Raspberry Pi.


#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

#[macro_use]
extern crate nix;
use nix::libc::c_int;
use nix::Result;
use std::ffi::CStr;
use std::fs::File;
use std::io;
use std::str;

use byteorder::{ByteOrder, LittleEndian};
use nix::errno::Errno;
use nix::errno::Errno::ENODEV;
use nix::Error::Sys;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::ErrorKind;
use std::io::SeekFrom;
use std::io::Write;
use std::iter;
use std::os::unix::io::AsRawFd;

#[allow(dead_code)]
mod ioctl;
pub mod picontrol;

/// RevPiControl is an object representing an open file handle to the piControl driver file descriptor.
pub struct RevPiControl {
    path: String,
    handle: Option<File>,
}

impl Default for picontrol::SDeviceInfo {
    fn default() -> picontrol::SDeviceInfo {
        unsafe { std::mem::zeroed() }
    }
}

impl Default for picontrol::SPIVariable {
    fn default() -> picontrol::SPIVariable {
        unsafe { std::mem::zeroed() }
    }
}

impl Default for picontrol::SPIValue {
    fn default() -> picontrol::SPIValue {
        unsafe { std::mem::zeroed() }
    }
}

fn byteToInt8Array(name: &str) -> [::std::os::raw::c_char; 32] {
    let i8slice = unsafe { &*(name.as_bytes() as *const [u8] as *const [::std::os::raw::c_char]) };
    let mut bname: [::std::os::raw::c_char; 32] = Default::default();
    let (left, _) = bname.split_at_mut(i8slice.len());
    left.copy_from_slice(i8slice);
    bname
}

// numToBytes converts a generic fixed-size value to its byte representation.
pub fn numToBytes(num: u64, size: usize) -> std::result::Result<Vec<u8>, Box<std::error::Error>> {
    match size {
        8 => return Ok(vec![num as u8]),
        16 => {
            let mut buf = [0; 2];
            LittleEndian::write_u16(&mut buf, num as u16);
            return Ok(buf.to_vec());
        }
        32 => {
            let mut buf = [0; 4];
            LittleEndian::write_u32(&mut buf, num as u32);
            return Ok(buf.to_vec());
        }
        64 => {
            let mut buf = [0; 8];
            LittleEndian::write_u64(&mut buf, num as u64);
            return Ok(buf.to_vec());
        }
        _ => return Err(From::from(format!("invalid size {}", size))),
    }
}

impl RevPiControl {

    pub fn new() -> Self {
        let c_str = CStr::from_bytes_with_nul(picontrol::PICONTROL_DEVICE).unwrap();
        let path: &str = c_str.to_str().unwrap();
        RevPiControl { handle: None, path: String::from(path) }
    }

    pub fn new_at(path: &str) -> Self {
        RevPiControl { handle: None, path: path.to_owned() }
    }

    /// Open the Pi Control interface. 
    pub fn open(&mut self) -> io::Result<bool> {
        if let Some(_) = self.handle.as_mut() {
            return Ok(true);
        }


        let file = OpenOptions::new().read(true).write(true).open(&self.path)?;
        self.handle = Some(file);
        Ok(true)
    }

    /// Close the Pi Control interface.
    pub fn close(&mut self) {
        if let Some(f) = self.handle.as_mut() {
            std::mem::drop(f);
            self.handle = None;
            return;
        }
    }

    /// Reset Pi Control Interface.
    pub fn reset(&self) -> Result<c_int> {
        let f = self.handle.as_ref().ok_or(Sys(ENODEV))?;
        return unsafe { ioctl::reset(f.as_raw_fd()) };
    }

    // Gets process data from a specific position, reads @length bytes from file.
    // Returns a result containing the bytes read or error.
    pub fn read(&mut self, offset: u64, length: usize) -> std::io::Result<(Vec<u8>)> {
        let f = self
            .handle
            .as_mut()
            .ok_or(io::Error::new(ErrorKind::NotFound, "error reading file"))?;
        /* seek */
        f.seek(SeekFrom::Start(offset))?;
        let mut v = vec![0u8; length];
        f.read_exact(&mut v)?;
        Ok(v)
    }

    /// Writes process data at a specific position and a returns a boolean result.
    pub fn write(&mut self, offset: u64, data: &Vec<u8>) -> std::io::Result<(bool)> {
        let f = self
            .handle
            .as_mut()
            .ok_or(io::Error::new(ErrorKind::NotFound, "error reading file"))?;
        /* seek */
        f.seek(SeekFrom::Start(offset))?;
        f.write(data)?;
        Ok(true)
    }

    /// Get the info for a variable.
    pub fn getVariableInfo(&self, name: &str) -> Result<picontrol::SPIVariable> {
        let f = self.handle.as_ref().ok_or(Sys(ENODEV))?;
        let mut v = picontrol::SPIVariable {
            strVarName: byteToInt8Array(name),
            ..Default::default()
        };
        let res = unsafe { ioctl::getVariableInfo(f.as_raw_fd(), &mut v) }?;
        if res < 0 {
            return Err(Sys(Errno::last()));
        }
        Ok(v)
    }

    /// Gets a description of connected devices.
    pub fn getDeviceInfoList(&self) -> Result<Vec<picontrol::SDeviceInfo>> {
        let f = self.handle.as_ref().ok_or(Sys(ENODEV))?;
        // let mut pDev: picontrol::SDeviceInfo = unsafe { mem::uninitialized() };
        let mut pDev = [picontrol::SDeviceInfo {
            ..Default::default()
        }; picontrol::REV_PI_DEV_CNT_MAX as usize];
        let res = unsafe { ioctl::getDeviceInfoList(f.as_raw_fd(), &mut pDev[0]) }?;
        if res < 0 {
            return Err(Sys(Errno::last()));
        }
        Ok(pDev[..res as usize].to_vec())
    }

    /// Gets the value of one bit in the process image.
    pub fn getBitValue(&self, pSpiValue: &mut picontrol::SPIValue) -> Result<bool> {
        return self.handleBitValue(pSpiValue, ioctl::getBitValue);
    }

    /// Sets the value of one bit in the process image.
    pub fn setBitValue(&self, pSpiValue: &mut picontrol::SPIValue) -> Result<bool> {
        return self.handleBitValue(pSpiValue, ioctl::setBitValue);
    }

    fn handleBitValue(
        &self,
        pSpiValue: &mut picontrol::SPIValue,
        func: unsafe fn(i32, *mut picontrol::SPIValueStr) -> std::result::Result<i32, nix::Error>,
    ) -> Result<bool> {
        let f = self.handle.as_ref().ok_or(Sys(ENODEV))?;

        pSpiValue.i16uAddress += (pSpiValue.i8uBit as u16) / 8;
        pSpiValue.i8uBit %= 8;

        let res = unsafe { func(f.as_raw_fd(), pSpiValue) }?;
        if res < 0 {
            return Err(Sys(Errno::last()));
        }
        Ok(true)
    }

    const SMALL_BUFFER_SIZE: usize = 256;
    const LARGE_BUFFER_SIZE: usize = 64 * 1024;

    /// dumps the process image to a file.
    /// 
    /// # Arguments
    ///
    /// * `fp` - The file path
    /// 
    pub fn dump(&mut self, fp: &str) -> std::io::Result<(bool)> {
        let f = self
            .handle
            .as_mut()
            .ok_or(io::Error::new(ErrorKind::NotFound, "error reading file"))?;
        /* seek */
        f.seek(SeekFrom::Start(0))?;

        let mut outfile = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(fp)?;
        // f.write(data)?;
        let buffer = &mut vec![0; Self::SMALL_BUFFER_SIZE];

        // We create a buffered writer from the file we get
        // let mut writer = BufWriter::new(&outfile);
        Self::redirect_stream(f, &mut outfile, buffer)?;
        Ok(true)
    }

    fn redirect_stream<R, W>(reader: &mut R, writer: &mut W, buffer: &mut Vec<u8>) -> io::Result<()>
    where
        R: Read,
        W: Write,
    {
        loop {
            let len_read = reader.read(buffer)?;

            if len_read == 0 {
                return Ok(());
            }

            writer.write_all(&buffer[..len_read])?;

            if len_read == buffer.len() && len_read < Self::LARGE_BUFFER_SIZE {
                buffer.extend(iter::repeat(0).take(len_read));
            }
        }
    }
}

impl Drop for RevPiControl {
    fn drop(&mut self) {
        self.close();
    }
}

// getModuleName returns a friendly name for a RevPi module type.
pub fn getModuleName(moduletype: u32) -> &'static str {
    let moduletype = moduletype & picontrol::PICONTROL_NOT_CONNECTED_MASK;
    let moddes = match moduletype {
        95 => "RevPi Core",
        96 => "RevPi DIO",
        97 => "RevPi DI",
        98 => "RevPi DO",
        103 => "RevPi AIO",
        picontrol::PICONTROL_SW_MODBUS_TCP_SLAVE => "ModbusTCP Slave Adapter",
        picontrol::PICONTROL_SW_MODBUS_RTU_SLAVE => "ModbusRTU Slave Adapter",
        picontrol::PICONTROL_SW_MODBUS_TCP_MASTER => "ModbusTCP Master Adapter",
        picontrol::PICONTROL_SW_MODBUS_RTU_MASTER => "ModbusRTU Master Adapter",
        100 => "Gateway DMX",
        71 => "Gateway CANopen",
        73 => "Gateway DeviceNet",
        74 => "Gateway EtherCAT",
        75 => "Gateway EtherNet/IP",
        93 => "Gateway ModbusTCP",
        76 => "Gateway Powerlink",
        77 => "Gateway Profibus",
        79 => "Gateway Profinet IRT",
        81 => "Gateway SercosIII",
        _ => "unknown moduletype",
    };

    moddes
}

// IsModuleConnected checks whether a RevPi module is conneted.
pub fn isModuleConnected(moduletype: u32) -> bool {
    return moduletype & picontrol::PICONTROL_NOT_CONNECTED > 0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picontrol_constants() {
        assert_eq!(picontrol::PICONTROL_DEVICE, b"/dev/piControl0\0");
    }

}
