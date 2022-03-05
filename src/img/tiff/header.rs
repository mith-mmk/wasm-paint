/* for EXIF */
use crate::img::error::{ImgError,ErrorKind};
use crate::img::error::ImgError::{SimpleAddMessage};
use super::tags::gps_mapper;
use super::tags::tag_mapper;
use super::super::util::*;
use super::super::io::*;


pub struct Rational {
    pub n: u32,
    pub d: u32,
}

pub struct RationalU64 {
    pub n: u64,
    pub d: u64,
}

pub enum DataPack {
    Bytes(Vec<u8>),
    Ascii(String),
    SByte(Vec<i8>),
    Short(Vec<u16>),
    Long(Vec<u32>),
    Rational(Vec<Rational>),
    RationalU64(Vec<RationalU64>),
    Float(Vec<f32>),
    Double(Vec<f64>),
    SShort(Vec<i16>),
    SLong(Vec<i32>),
    Unkown(Vec<u8>),
    Undef(Vec<u8>),
}

pub fn print_data (data: &DataPack) {
    match data {
        DataPack::Rational(d) => {
            debug_print!("{} ",d.len());
            for i in 0..d.len() {
                debug_print!("{}/{} ",d[i].n,d[i].d)
            }
        },
        DataPack::RationalU64(d) => {
            debug_print!("{} ",d.len());
            for i in 0..d.len() {
                debug_print!("{}/{} ",d[i].n,d[i].d)
            }
        },
        DataPack::Bytes(d) => {
            for i in 0..d.len() {
                debug_print!("{} ",d[i])
            }
        },
        DataPack::SByte(d) => {
            for i in 0..d.len() {
                debug_print!("{} ",d[i])
            }
        },
        DataPack::Undef(d) => {
            for i in 0..d.len() {
                debug_print!("{} ",d[i])
            }
        },
        DataPack::Ascii(s) => {
            debug_print!("{}", s)
        },
        DataPack::Short(d) => {
            for i in 0..d.len() {
                debug_print!("{} ",d[i])
            }
        },
        DataPack::Long(d) => {
            for i in 0..d.len() {
                debug_print!("{} ",d[i])
            }
        },
        DataPack::SShort(d) => {
            for i in 0..d.len() {
                debug_print!("{} ",d[i])
            }
        },
        DataPack::SLong(d) => {
            for i in 0..d.len() {
                debug_print!("{} ",d[i])
            }
        },
        DataPack::Float(d) => {
            for i in 0..d.len() {
                debug_print!("{} ",d[i])
            }
        },
        DataPack::Double(d) => {
            for i in 0..d.len() {
                debug_print!("{} ",d[i])
            }
        },
        _ => {

        },
    }
    debug_println!();
}

#[allow(unused)]
pub struct TiffHeader {
    tagid: usize,
    data: DataPack,
}

#[allow(unused)]
pub struct TiffHeaders {
    headers :Vec<TiffHeader>,
    exif: Option<Vec<TiffHeader>>,
    gps: Option<Vec<TiffHeader>>,
    little_endian: bool,
}

pub fn read_tags( buffer: &Vec<u8>) -> Result<TiffHeaders,ImgError>{
    let endian :bool;
    if buffer[0] != buffer [1] {
        return Err(SimpleAddMessage(ErrorKind::IlligalData,"not Tiff".to_string()));
    }

    if buffer[0] == 'I' as u8 { // Little Endian
        endian = true;
        debug_println!("Little Endian"); 
    } else if buffer[0] == 'M' as u8 {      // Big Eindian
        endian = false;
        debug_println!("Big Endian"); 
    } else {
        debug_println!("not TIFF");
        return Err(SimpleAddMessage(ErrorKind::IlligalData,"not Tiff".to_string()));
    }

    let mut ptr = 2 as usize;
    // version
    let ver = read_u16(buffer,ptr,endian);
    ptr = ptr + 2;
    let offset_ifd  = read_u32(buffer,ptr,endian) as usize;
    debug_println!("Tiff Version {:>08}",ver);
    read_tiff(buffer,offset_ifd,endian)
}

fn get_data (buffer: &[u8], ptr :usize ,datatype:usize, datalen: usize, endian: bool) -> DataPack {
    let data :DataPack;
    match datatype {
        1  => {  // 1.BYTE(u8)
            let mut d: Vec<u8> = Vec::with_capacity(datalen);
            if datalen <=4 {
                for i in 0.. datalen { 
                    d.push(read_byte(buffer,ptr + i));
                }
            } else {
                let offset = read_u32(buffer,ptr,endian) as usize;
                for i in 0.. datalen { 
                    d.push(read_byte(buffer,offset + i));
                }
            }
            data = DataPack::Bytes(d);
        },
        2 => {  // 2. ASCII(u8)
            let string;
            if datalen <=4 {
                string = read_string(buffer,ptr,datalen);

            } else {
                let offset = read_u32(buffer,ptr,endian) as usize;
                string = read_string(buffer,offset,datalen);
            }
            data = DataPack::Ascii(string);    
        }
        3 => {  // SHORT (u16)
            let mut d: Vec<u16> = Vec::with_capacity(datalen);
            if datalen*2 <= 4 {
                if datalen == 1 {
                    d.push(read_u16(buffer,ptr,endian));
                } else if datalen == 2{
                    d.push(read_u16(buffer,ptr,endian));
                    d.push(read_u16(buffer,ptr + 2,endian));
                }
            } else {
                let offset = read_u32(buffer,ptr,endian) as usize;
                for i in 0.. datalen { 
                    d.push(read_u16(buffer,offset + i*2,endian));
                }
            }
            data = DataPack::Short(d);
        },
        4 => {  // LONG (u32)
            let mut d :Vec<u32> = Vec::with_capacity(datalen);
            if datalen*4 <= 4 {
                d.push(read_u32(buffer,ptr,endian));
            } else {
                let offset = read_u32(buffer,ptr,endian) as usize;
                for i in 0.. datalen { 
                    d.push(read_u32(buffer,offset + i*4,endian));
                }
            }
            data = DataPack::Long(d);
        },
        5 => {  //RATIONAL u32/u32
            let mut d :Vec<Rational> = Vec::with_capacity(datalen);
            let offset = read_u32(buffer,ptr,endian) as usize;
            for i in 0.. datalen { 
                let n  = read_u32(buffer,offset + i*8,endian);
                let denomi = read_u32(buffer,offset + i*8+4,endian);
                d.push(Rational{n:n,d:denomi});

            }
            data = DataPack::Rational(d);
        },
        6 => {  // 6 i8 
            let mut d: Vec<i8> = Vec::with_capacity(datalen);
            if datalen <=4 {
                for i in 0.. datalen { 
                    d.push(read_i8(buffer,ptr + i));
                }
            } else {
                let offset = read_u32(buffer,ptr,endian) as usize;
                for i in 0.. datalen { 
                    d.push(read_i8(buffer,offset + i));

                }
            }
            data = DataPack::SByte(d);
        },
        7 => {  // 1.undef
            let mut d: Vec<u8> = Vec::with_capacity(datalen);
            if datalen <=4 {
                for i in 0.. datalen { 
                    d.push(read_byte(buffer,ptr + i));
                }
            } else {
                let offset = read_u32(buffer,ptr,endian) as usize;
                for i in 0.. datalen { 
                    d.push(read_byte(buffer,offset + i));
                }
            }
            data = DataPack::Undef(d);
        },
        8 => {  // 6 i8 
            let mut d: Vec<i16> = Vec::with_capacity(datalen);
            if datalen <=4 {
                for i in 0.. datalen { 
                    d.push(read_i16(buffer,ptr + i,endian));
                }
            } else {
                let offset = read_u32(buffer,ptr,endian) as usize;
                for i in 0.. datalen { 
                    d.push(read_i16(buffer,offset + i,endian));

                }
            }
            data = DataPack::SShort(d);
        },
        9 => {  // i32 
            let mut d: Vec<i32> = Vec::with_capacity(datalen);
            if datalen <=4 {
                for i in 0.. datalen { 
                    d.push(read_i32(buffer,ptr + i,endian));
                }
            } else {
                let offset = read_u32(buffer,ptr,endian) as usize;
                for i in 0.. datalen { 
                    d.push(read_i32(buffer,offset + i,endian));

                }
            }
            data = DataPack::SLong(d);
        },
        // 7 undefined 8 s16 9 s32 10 srational u64/u64 11 float 12 double
        10 => {  //RATIONAL u64/u64
            let mut d :Vec<RationalU64> = Vec::with_capacity(datalen);
            let offset = read_u32(buffer,ptr,endian) as usize;
            for i in 0.. datalen { 
                let n_u64 = read_u64(buffer,offset + i*8,endian);
                let d_u64 =read_u64(buffer,offset + i*8+4,endian);
                d.push(RationalU64{n:n_u64,d:d_u64});
            }
            data = DataPack::RationalU64(d);

        },
        11 => {  // f32 
            let mut d: Vec<f32> = Vec::with_capacity(datalen);
            if datalen <=4 {
                for i in 0.. datalen { 
                    d.push(read_f32(buffer,ptr + i,endian));
                }
            } else {
                let offset = read_u32(buffer,ptr,endian) as usize;
                for i in 0.. datalen { 
                    d.push(read_f32(buffer,offset + i,endian));

                }
            }
            data = DataPack::Float(d);
        },
        12 => {  // f64 
            let mut d: Vec<f64> = Vec::with_capacity(datalen);
            if datalen <=4 {
                for i in 0.. datalen { 
                    d.push(read_f64(buffer,ptr + i,endian));
                }
            } else {
                let offset = read_u32(buffer,ptr,endian) as usize;
                for i in 0.. datalen { 
                    d.push(read_f64(buffer,offset + i,endian));

                }
            }
            data = DataPack::Double(d);
        },
        _ => {
            debug_println!("Unknown Data type {}",datatype);
            let mut d: Vec<u8> = Vec::with_capacity(datalen);
            if datalen <=4 {
                for i in 0.. datalen { 
                    d.push(read_byte(buffer,ptr + i));
                }
            } else {
                let offset = read_u32(buffer,ptr,endian) as usize;
                for i in 0.. datalen { 
                    d.push(read_byte(buffer,offset + i));
                }
            };
            data = DataPack::Unkown(d);
        }
    }
    data
}

fn read_tiff (buffer: &[u8], offset_ifd: usize,endian: bool) -> Result<TiffHeaders,ImgError>{
    read_tag(buffer,offset_ifd,endian,0)
}

fn read_gps (buffer: &[u8], offset_ifd: usize,endian: bool) -> Result<TiffHeaders,ImgError> {
    read_tag(buffer,offset_ifd,endian,2)
}

fn read_tag (buffer: &[u8], mut offset_ifd: usize,endian: bool,mode: usize) -> Result<TiffHeaders,ImgError>{
    let mut ifd = 0;
    let mut headers :TiffHeaders = TiffHeaders{headers:Vec::new(),exif:None,gps:None,little_endian: endian};
    loop {
        debug_println!("{}{} {}",if mode == 2 {"GPS"} else if mode == 1 {"EXIF"} else {"IFD"} ,ifd,offset_ifd);    
        let mut ptr = offset_ifd;
        let tag = read_u16(buffer,ptr,endian);
        ptr = ptr + 2;
        debug_println!("{} tag haves",tag);
 
        for _ in 0..tag {
            let tagid = read_u16(buffer,ptr,endian);
            let datatype = read_u16(buffer,ptr + 2,endian) as usize;
            let datalen = read_u32(buffer,ptr + 4,endian) as usize;
            ptr = ptr + 8;
            debug_println!("ID {:>04X} datatype {} length {}",tagid,datatype,datalen);
            let data :DataPack = get_data(buffer, ptr, datatype, datalen, endian);
            ptr = ptr + 4;

    
            if mode != 2 {
                match tagid {
                    0x8769 => {
                        match &data {
                            DataPack::Long(d) => {
                                debug_println!("Exif Offset: {}", d[0]);
                                let r = read_tag(buffer, d[0] as usize, endian,1)?; // read exif
                                headers.exif = Some(r.headers);

                            },
                            _  => {
                            }
                        }
                    },
                    0x8825 => {
                        match &data {
                            DataPack::Long(d) => {
                                debug_println!("GPS TAG: {}",d[0]);
                                let r = read_gps(buffer, d[0] as usize, endian)?; // read exif
                                headers.gps = Some(r.headers);
                        },
                        _  => {
                        }
                        }
                    },
                    _ => {
                        #[cfg(debug_assertions)]
                        tag_mapper(tagid ,&data);
                    }
                }
            } else {
                gps_mapper(tagid ,&data);
            }
            headers.headers.push(TiffHeader{tagid: tagid as usize,data: data});
        }
        offset_ifd  = read_u32(buffer,ptr,endian) as usize;
        if offset_ifd == 0 {break ;}
        ifd = ifd + 1;
    }
    Ok(headers)
}
