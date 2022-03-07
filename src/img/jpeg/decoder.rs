use core::f64::consts::PI;
use crate::img::jpeg::header::Component;
use crate::img::jpeg::header::HuffmanTable;
use crate::img::jpeg::header::JpegHaeder;
use crate::img::jpeg::util::print_header;
use crate::img::jpeg::worning::JPEGWorning::SimpleAddMessage as WorningAddMessage;
use crate::img::jpeg::worning::JPEGWorning;
use crate::img::jpeg::worning::WorningKind;
use crate::img::error::ImgError::SimpleAddMessage;
use crate::img::error::{ImgError,ErrorKind};
use crate::img::error::ImgError::{Simple};
use crate::img::DecodeOptions;

use crate::log;


#[allow(unused)]
#[cfg(not(debug_assertions))]
macro_rules! huff_print {
    ($l:expr,$code:expr,$number:expr,$pos:expr) => {
    };
}

#[allow(unused)]
#[cfg(debug_assertions)]
macro_rules! huff_print {
    ($l:expr,$code:expr,$number:expr,$pos:expr) => {
        
        match $l {
            2 => {
                debug_println!("number {:02x}  code {:>02b} pos{}",$number,$code,$pos);
            },
            3 => {
                debug_println!("number {:02x}  code {:>03b} pos{}",$number,$code,$pos);
            },
            4 => {
                debug_println!("number {:02x}  code {:>04b} pos{}",$number,$code,$pos);
            },
            5 => {
                debug_println!("number {:02x}  code {:>05b} pos{}",$number,$code,$pos);
            },
            6 => {
                debug_println!("number {:02x}  code {:>06b} pos{}",$number,$code,$pos);
            },
            7 => {
                debug_println!("number {:02x}  code {:>07b} pos{}",$number,$code,$pos);
            },
            8 => {
                debug_println!("number {:02x}  code {:>08b} pos{}",$number,$code,$pos);
            },
            9 => {
                debug_println!("number {:02x}  code {:>09b} pos{}",$number,$code,$pos);
            },
            _ => {
                debug_println!("number {:02x}  code {:>b} pos{}",$number,$code,$pos);
            },
        }
    };
}


struct BitReader {
    buffer: Vec<u8>,
    ptr : usize,
    bptr : usize,
    b: u8,
    rst: bool,
    rst_ptr : usize,
    eof_flag: bool,
}

#[allow(unused)]
impl BitReader {
    pub fn new(buffer:&[u8]) -> Self{
        let ptr:usize = 0;
        let bptr:usize = 0;
        let b:u8 = 0;
        Self{
            buffer: buffer.to_vec(),
            ptr: ptr,
            bptr: bptr,
            b: b,
            rst: false,
            rst_ptr: 0,
            eof_flag: false,
        }
    }

    fn get_byte(self: &mut Self) -> Result<u8,ImgError> {
        if self.ptr >= self.buffer.len() {
            return Err(Simple(ErrorKind::OutboundIndex));
        }
        self.b = self.buffer[self.ptr];
//        println!("{:>04X} {:>02x} {:08b} ",self.ptr,self.b,self.b);
        self.ptr = self.ptr + 1;
        Ok(self.b)
    }

    fn rst(self: &mut Self) -> Result<bool,ImgError> {
        Ok(self.rst)
    }

    pub fn get_bit_as_i32(self: &mut Self) -> Result<i32,ImgError> {
        if self.bptr == 0 {
            self.bptr = 8;
            if self.get_byte()? == 0xff {
                match self.get_byte()? {
                    0x00 => {
                        self.b = 0xff;
                    },
                    0xd0..=0xd7 =>  {    // RST    
                        self.rst = true; 
                        self.rst_ptr = self.ptr;
                        self.b = 0xff;
                    },
                    0xd9=> { // EOI
                        self.b = 0xff;
                    },
                    _ =>{
//                        log(&format!("{:>02x}",self.b));
                        self.b = 0xff;
                        return Err(SimpleAddMessage(ErrorKind::DecodeError,"FF after  00 or RST".to_string()))
                    },                    
                }
            }
        }
        self.bptr = self.bptr - 1;
        let r:i32 = (self.b  >> self.bptr) as i32 & 0x1;
        Ok(r)
    }

    pub fn eof(self: &mut Self) -> bool {
        if self.buffer.len() - 2 < self.ptr || self.eof_flag
         {println!("eof {}",self.buffer.len());self.eof_flag=true}
        self.eof_flag
    }

    pub fn flush(self: &mut Self) {
        if self.rst == true {
            self.rst =false;
            self.bptr = 0;
//            self.ptr = self.rst_ptr;
        }
    }

    pub fn reset(self: &mut Self) {
        self.ptr = 0;
        self.eof_flag = false;
    }

    pub fn set_offset(self: &mut Self ,offset: usize) -> Result<usize,ImgError> {
        if offset < self.buffer.len() {
            self.ptr = offset;
            self.eof_flag = false;
            Ok(self.ptr)
        } else {
            Err(Simple(ErrorKind::OutboundIndex))
        }
    }

    pub fn offset(self: &mut Self) -> usize {
        self.ptr
    }

}

fn huffman_read (bit_reader:&mut BitReader,table: &HuffmanDecodeTable)  -> Result<u32,ImgError>{
    let mut v = 0;
    let mut d :i32 = 0;
    let mut ll = 1;
    for l in 0..16 {
        d = (d << 1) | bit_reader.get_bit_as_i32()?;
        if table.max[l] >= d {
            let p = d as usize - table.min[l] as usize + table.pos[l] as usize;
            v = table.val[p];
                        
            break;
        }
        ll = ll + 1;
    }
    Ok(v as u32)
}


#[derive(std::cmp::PartialEq)]
pub struct HuffmanDecodeTable {
    pos: Vec::<usize>,
    val: Vec::<usize>,
    min: Vec::<i32>,
    max: Vec::<i32>,     
}

#[inline]
fn dc_read(bitread: &mut BitReader,dc_decode:&HuffmanDecodeTable,pred:i32) -> Result<i32,ImgError> {
    let ssss = huffman_read(bitread,&dc_decode)?;
    let v =  receive(bitread,ssss as i32)?;
    let diff = extend(v,ssss as i32);
    let dc = diff + pred;
    Ok(dc)
}

#[inline]
fn ac_read(bitread: &mut BitReader,ac_decode:&HuffmanDecodeTable) -> Result<Vec<i32>,ImgError> {
    let mut zigzag : usize= 1;
    let mut zz :Vec<i32> = (0..64).map(|_| 0).collect();
    loop {  // F2.2.2
        let ac = huffman_read(bitread,&ac_decode)?;
        
        let ssss = ac & 0xf;
        let rrrr = ac >> 4;
        if ssss == 0 {
            if ac == 0x00 { //EOB
                return Ok(zz)
            }
            if rrrr == 15 { //ZRL
                zigzag = zigzag + 16;
                continue
            }
            return Ok(zz)   // N/A
        } else {
            zigzag = zigzag + rrrr as usize;
            let v =  receive(bitread,ssss as i32)?;
            zz[zigzag] = extend(v,ssss as i32);
        }
        if zigzag >= 63 {
            return Ok(zz)
        }
        zigzag = zigzag + 1;
    }
}

#[inline]
fn baseline_read(bitread: &mut BitReader,dc_decode:&HuffmanDecodeTable,ac_decode:&HuffmanDecodeTable,pred: i32)-> Result<(Vec<i32>,i32),ImgError> {
    let dc = dc_read(bitread, dc_decode, pred)?;
    let mut zz = ac_read(bitread, ac_decode)?;
    zz[0] = dc;
    Ok((zz,dc))
}

#[inline]
fn receive(bitread: &mut BitReader, ssss :i32) -> Result<i32,ImgError>{
  let mut v = 0;

  for _ in 0..ssss {
    v = (v << 1) + bitread.get_bit_as_i32()?;
  }
  Ok(v)
}

#[inline]
fn extend(mut v:i32,t: i32) -> i32 {
    if t == 0 {
        return v;
    }
    let mut vt = 1 << (t-1);

    if v < vt {
        vt = (-1 << t) + 1;
        v = v + vt;
    }
    v
}

#[inline]
fn idct(f :&[i32]) -> Vec<u8> {
    let vals :Vec<u8> = (0..64).map(|i| {
        let (x,y) = ((i%8) as f64,(i/8) as f64);
        // IDCT from CCITT Rec. T.81 (1992 E) p.27 A3.3
        let mut val: f64=0.0;
        for u in 0..8 {
            let cu = if u == 0 {1.0_f64 / 2.0_f64.sqrt()} else {1.0};
            for v in 0..8 {
                let cv = if v == 0 {1.0_f64 / 2.0_f64.sqrt()} else {1.0};
                val += cu * cv * (f[v*8 + u] as f64)
                    * ((2.0 * x + 1.0) * u as f64 * PI / 16.0_f64).cos()
                    * ((2.0 * y + 1.0) * v as f64 * PI / 16.0_f64).cos();
            }
        }
        val = val / 4.0;

        // level shift from CCITT Rec. T.81 (1992 E) p.26 A3.1
        let v = val.round() as i32 + 128;
        if v < 0 {0} else if v > 255 {255} else {v as u8}
    }).collect();
    vals
}

// Glayscale
fn y_to_rgb  (yuv: &Vec<Vec<u8>>,hv_maps:&Vec<Component>) -> Vec<u8> {
    let mut buffer:Vec<u8> = (0 .. hv_maps[0].h * hv_maps[0].v * 64 * 4).map(|_| 0).collect();
    for v in 0..hv_maps[0].v {
        for h in 0..hv_maps[0].h {
            let gray = &yuv[v*hv_maps[0].h + h];
            for y in 0..8 {
                let offset = (y + v *8) * hv_maps[0].h * 8 * 4;
                for x in 0..8 {
                    let xx = (x + h * 8) * 4;
                    let cy = gray[y * 8 + x];
                    buffer[xx + offset    ] = cy;   // R
                    buffer[xx + offset + 1] = cy;   // G
                    buffer[xx + offset + 2] = cy;   // B
                    buffer[xx + offset + 3] = 0xff; // A
                }
            }
        }
    }
    buffer
}

fn yuv_to_rgb (yuv: &Vec<Vec<u8>>,hv_maps:&Vec<Component>) -> Vec<u8> {
    let mut buffer:Vec<u8> = (0..hv_maps[0].h * hv_maps[0].v * 64 * 4).map(|_| 0).collect();
    let y_map = 0;
    let u_map = y_map + hv_maps[0].h * hv_maps[0].v;
    let v_map = u_map + hv_maps[1].h * hv_maps[1].v;

    let uy = hv_maps[0].v / hv_maps[1].v as usize;
    let vy = hv_maps[0].v / hv_maps[2].v as usize;
    let ux = hv_maps[0].h / hv_maps[1].h as usize;
    let vx = hv_maps[0].h / hv_maps[2].h as usize;

    for v in 0..hv_maps[0].v {
        let mut u_map_cur = u_map + v / hv_maps[0].h;
        let mut v_map_cur = v_map + v / hv_maps[0].h;

        for h in 0..hv_maps[0].h {
            let gray = &yuv[v*hv_maps[0].h + h];
            u_map_cur = u_map_cur + h / hv_maps[0].h;
            v_map_cur = v_map_cur + h / hv_maps[0].h;

            for y in 0..8 {
                let offset = ((y + v * 8) * (8 * hv_maps[0].h)) * 4;
                for x in 0..8 {
                    let xx = (x + h * 8) * 4;
                    let cy = gray[y * 8 + x] as f32;
                    let cb = yuv[u_map_cur][(((y + v * 8) / uy % 8) * 8)  + ((x + h * 8) / ux) % 8] as f32;
                    let cr = yuv[v_map_cur][(((y + v * 8) / vy % 8) * 8)  + ((x + h * 8) / vx) % 8] as f32;

                    let red  = cy as f32 + 1.402 * (cr - 128.0);
                    let green= cy as f32 - 0.34414 * (cb - 128.0) - 0.71414 * (cr - 128.0);
                    let blue = cy as f32 + 1.772 * (cb - 128.0);

                    let red = if red > 255.0 {255} else if red < 0.0 {0} else {red as u8};
                    let green = if green > 255.0 {255} else if green < 0.0 {0} else {green as u8};
                    let blue = if blue > 255.0 {255} else if blue < 0.0 {0} else {blue as u8};

                    buffer[xx + offset    ] = red; //R
                    buffer[xx + offset + 1] = green; //G
                    buffer[xx + offset + 2] = blue; //B
                    buffer[xx + offset + 3] = 0xff; //A
                }
            }
        }
    }

    buffer
}

pub fn huffman_extend(huffman_tables:&Vec<HuffmanTable>) -> (Vec<HuffmanDecodeTable>,Vec<HuffmanDecodeTable>) {

    let mut ac_decode : Vec<HuffmanDecodeTable> = Vec::new();
    let mut dc_decode : Vec<HuffmanDecodeTable> = Vec::new();

    for huffman_table in huffman_tables.iter() {

        let mut current_max: Vec<i32> = Vec::new();
        let mut current_min: Vec<i32> = Vec::new();

        let mut code :i32 = 0;
        let mut pos :usize = 0;
        for l in 0..16 {
            if huffman_table.len[l] != 0 {
                current_min.push(code); 
                for _ in 0..huffman_table.len[l] {
                    if pos >= huffman_table.val.len() { break;}
                    pos = pos + 1;
                    code = code + 1;
                }
                current_max.push(code - 1); 
            } else {
                current_min.push(-1);
                current_max.push(-1);
            }
            code = code << 1;
        }
        
        if huffman_table.ac {
            let val : Vec<usize> = huffman_table.val.iter().map(|i| *i).collect();
            let pos : Vec<usize> = huffman_table.pos.iter().map(|i| *i).collect();
            ac_decode.push(HuffmanDecodeTable{
                val: val,
                pos: pos,
                max: current_max,
                min: current_min,
            });
        } else {
            let val : Vec<usize> = huffman_table.val.iter().map(|i| *i).collect();
            let pos : Vec<usize> = huffman_table.pos.iter().map(|i| *i).collect();
            dc_decode.push(HuffmanDecodeTable{
                val: val,
                pos: pos,
                max: current_max,
                min: current_min,
            });
        }
    }

    (ac_decode,dc_decode)
}

pub fn decode<'decode>(buffer: &[u8],option:&mut DecodeOptions) 
    -> Result<Option<JPEGWorning>,ImgError> {

    let mut worning: Option<JPEGWorning> = None;

     // Scan Header
    let header = JpegHaeder::new(buffer,0)?;

    
    match header.huffman_scan_header {
        None => {
            return Err(SimpleAddMessage(ErrorKind::DecodeError,"Not undefined Huffman Scan Header".to_string()));
        },
        _ => {

        }
    }
    let huffman_scan_header  = header.huffman_scan_header.as_ref().unwrap();
    match header.huffman_tables {
        None => {
            return Err(SimpleAddMessage(ErrorKind::DecodeError,"Not undefined Huffman Tables".to_string()));
        },
        _ => {

        }
    }

    match header.frame_header {
        None => {
            return Err(SimpleAddMessage(ErrorKind::DecodeError,"Not undefined Frame Header".to_string()));
        },
        _ => {

        }
    }

    let fh = header.frame_header.as_ref().unwrap();
    let width = fh.width;
    let height = fh.height;
    let plane = fh.plane;
    match fh.component {
        None => {
            return Err(SimpleAddMessage(ErrorKind::DecodeError,"Not undefined Frame Header Component".to_string()));
        },
        _ => {

        }
    }

    let component = fh.component.as_ref().unwrap();
    match header.quantization_tables {
        None => {
            return Err(SimpleAddMessage(ErrorKind::DecodeError,"Not undefined Quantization Tables".to_string()));
        },
        _ => {

        }
    }
    let quantization_tables = header.quantization_tables.as_ref().unwrap();

    if fh.huffman == false {
        return Err(SimpleAddMessage(ErrorKind::DecodeError,"This decoder suport huffman only".to_string()));
    }

    if fh.baseline == false {
        return Err(SimpleAddMessage(ErrorKind::DecodeError,"This Decoder support Baseline Only".to_string()));
    }

    if fh.differential == true {
        return Err(SimpleAddMessage(ErrorKind::DecodeError,"This Decoder not support differential".to_string()));
    }

    // Make Huffman Table

    let (ac_decode,dc_decode) = huffman_extend(&header.huffman_tables.as_ref().unwrap());

    if option.debug_flag > 0 {
        let boxstr = print_header(&header,option.debug_flag);
        log(&boxstr);
    }

    log("Decode Start");

    // decode
    (option.callback.init)(option.drawer,width,height)?;


    let slice = &buffer[header.imageoffset..];
    let bitread :&mut BitReader = &mut BitReader::new(&slice);
    let mut dy = 8;
    let mut dx = 8;
    let mut scan : Vec<(usize,usize,usize,usize)> = Vec::new();
    let mcu_size = {
        let mut size = 0;
        for i in 0..component.len() {
            size = size + component[i].h * component[i].v;
            let tq = component[i].tq;
            for _ in 0..component[i].h * component[i].v {
                scan.push((huffman_scan_header.tdcn[i],
                            huffman_scan_header.tacn[i],
                            i,tq));
            } 

            dx = usize::max(component[i].h * 8 ,dx);
            dy = usize::max(component[i].v * 8 ,dx);
        }
        size
    };

    let mut preds: Vec::<i32> = (0..component.len()).map(|_| 0).collect();

    let mcu_y =(height+dy-1)/dy;
    let mcu_x =(width+dx-1)/dx;

    let mut mcu_interval = if header.interval > 0 { header.interval as isize} else {-1};


    for y in 0..mcu_y {
        for x in 0..mcu_x {
            let mut yuv :Vec<Vec<u8>> = Vec::new();
            for scannumber in 0..mcu_size {
                let (dc_current,ac_current,i,tq) = scan[scannumber];
                log(&format!("mcu dc{} ac{} tq {}",dc_current,ac_current,tq));
                let ret = baseline_read(bitread
                            ,&dc_decode[dc_current]
                            ,&ac_decode[ac_current]
                            ,preds[i]);
                let (zz,pred);
                match ret {
                    Ok((_zz,_pred)) => {
                        zz = _zz;
                        pred = _pred; 
                    }
                    Err(r) => {
                        log(&r.fmt());
                        return Ok(Some(WorningAddMessage(WorningKind::DataCorruption,r.fmt())));
                    }
                }
                preds[i] = pred;

                let sq = &super::util::ZIG_ZAG_SEQUENCE;
                let zz :Vec<i32> = (0..64).map(|i| 
                    zz[i] * quantization_tables[tq].q[i] as i32).collect();
                let zz :Vec<i32> = (0..64).map(|i| zz[sq[i]]).collect();
                let ff = idct(&zz);
                yuv.push(ff);
            }

            let data = if plane == 3 {yuv_to_rgb(&yuv,&component)} else {y_to_rgb(&yuv,&component)};

            (option.callback.draw)(option.drawer,x*dx,y*dy,dx,dy,&data)?;

            if mcu_interval > 0 {
                if bitread.rst()? == true {
                    worning = Some(WorningAddMessage(WorningKind::IlligalRSTMaker,"mismatch mcu interval".to_string()));
                    bitread.flush();
                    for i in 0..preds.len() {
                        preds[i] = 0;
                    }
//                    mcu_interval = header.interval as isize;
                    continue;
                }
            } else if mcu_interval == 0 && header.interval != 0 {
                mcu_interval = header.interval as isize;
                bitread.flush();
                for i in 0..preds.len() {
                    preds[i] = 0;
                }
            } 
            mcu_interval = mcu_interval - 1;
        }
    }
    (option.callback.terminate)(option.drawer)?;
    Ok(worning)
}
