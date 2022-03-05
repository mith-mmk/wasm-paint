use crate::img::jpeg::header::Component;

use core::f64::consts::PI;
use crate::img::DefaultCallback;
use crate::img::jpeg::header::HuffmanTable;
use crate::img::error::ImgError::SimpleAddMessage;
use crate::ImageBuffer;
use crate::img::jpeg::header::JpegHaeder;
use crate::img::error::{ImgError,ErrorKind};
use crate::img::error::ImgError::{Simple};
use crate::img::DecodeOptions;



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

    pub fn get_bit_as_i32(self: &mut Self) -> Result<i32,ImgError> {
        if self.bptr == 0 {
            self.bptr = 8;
            if self.get_byte()? == 0xff {
                self.ptr = self.ptr + 1;
                self.b = 0xff;
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
        self.bptr = 0;
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

struct BitReader {
    buffer: Vec<u8>,
    ptr : usize,
    bptr : usize,
    b: u8,
    eof_flag: bool,
}

struct HuffmanDecodeTable<'a> {
    pos: &'a Vec::<usize>,
    val: &'a Vec::<usize>,
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
    loop {
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
        let mut val: f64=0.0;
        let (x,y) = ((i%8) as f64,(i/8) as f64);
        // IDCT from CCITT Rec. T.81 (1992 E) p.27
        for u in 0..8 {
            for v in 0..8 {
                let cu = if u == 0 {1.0_f64 / 2.0_f64.sqrt()} else {1.0};
                let cv = if v == 0 {1.0_f64 / 2.0_f64.sqrt()} else {1.0};
                val += cu * cv * (f[u*8 + v] as f64)
                    * ((2.0 * x + 1.0) * u as f64 * PI / 16.0_f64).cos()
                    * ((2.0 * y + 1.0) * v as f64 * PI / 16.0_f64).cos();
            }
        }
        val = val / 4.0;

        // level shift from CCITT Rec. T.81 (1992 E) p.26
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
    let mut u_map = y_map + hv_maps[0].h * hv_maps[0].v;
    let mut v_map = u_map + hv_maps[1].h * hv_maps[1].v;

    println!("{} {} {} {} {} {}",yuv[0][0],yuv[1][0],yuv[2][0],yuv[3][0],yuv[4][0],yuv[5][0]);

    for v in 0..hv_maps[0].v {
        let mut uy = v * 8 * (hv_maps[1].v / hv_maps[0].v) as usize;
        if uy >= 8 {
            u_map = u_map + 1;
            uy = 0;
        }
        let mut vy = v * 8 * (hv_maps[2].v / hv_maps[0].v) as usize;
        if vy >= 8 {
            v_map = v_map + 1;
            vy = 0;
        }
        for h in 0..hv_maps[0].h {
            let gray = &yuv[v*hv_maps[0].h + h];
            let mut ux = (h * 8 ) * (hv_maps[1].h / hv_maps[0].h) as usize;
            if ux >= 8 {
                u_map = u_map + 1;
                ux = 0;
            }
            let mut vx = (h * 8 ) * (hv_maps[2].h / hv_maps[0].h) as usize;
            if vx >= 8 {
                v_map = v_map + 1;
                vx = 0;
            }
            for y in 0..8 {
                let offset = ((y + v * 8) * (8 * hv_maps[0].h)) * 4;
                for x in 0..8 {
                    let xx = (x + h * 8) * 4;
                    let cy = gray[y * 8 + x] as f32;
                    let cu = yuv[u_map][uy * 8 + ux +  (hv_maps[0].h / hv_maps[1].h)] as f32;
                    let cv = yuv[v_map][vy * 8 + vx +  (hv_maps[0].h / hv_maps[2].h)] as f32;

                    let red  = ((cy as f32 + 1.402 * (cu - 128.0)) as usize & 0xff) as u8;
                    let green= ((cy as f32 + 0.34414 * (cv - 128.0) - 0.71414 * (cu - 128.0)) as usize & 0xff) as u8;
                    let blue = ((cy as f32 + 1.772 * (cv - 128.0)) as usize &0xff) as u8;

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


pub fn decode(buffer: &[u8],option:&mut DecodeOptions) 
    -> Result<Option<ImageBuffer>,ImgError> {

// Scan Header
    let header = JpegHaeder::new(buffer,0)?;

    let huffman_scan_header  = header.huffman_scan_header.as_ref().unwrap();
    let huffman_tables = header.huffman_tables.as_ref().unwrap();
    let fh = header.frame_header.as_ref().unwrap();
    let width = fh.width;
    let height = fh.height;
    let plane = fh.plane;
    let component = fh.component.as_ref().unwrap();
    let quantization_tables = header.quantization_tables.unwrap();

    if fh.huffman == false {
        return Err(SimpleAddMessage(ErrorKind::DecodeError,"This decoder suport huffman only".to_string()));
    }

    if fh.baseline == false {
        return Err(SimpleAddMessage(ErrorKind::DecodeError,"This Decoder support Baseline Only".to_string()));
    }

    // Make Huffman Table
    let mut ac_decode : Vec<Option<HuffmanDecodeTable>> = (0..4).map(|_| None).collect();
    let mut dc_decode : Vec<Option<HuffmanDecodeTable>> = (0..4).map(|_| None).collect();

    for i in 0..huffman_tables.len() {
        let huffman_table :&HuffmanTable = &huffman_tables[i];

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
            ac_decode[huffman_table.no] = Some(HuffmanDecodeTable{
                val: &huffman_table.val,
                pos: &huffman_table.pos,
                max: current_max,
                min: current_min,
            })
        } else {
            dc_decode[huffman_table.no] = Some(HuffmanDecodeTable{
                val: &huffman_table.val,
                pos: &huffman_table.pos,
                max: current_max,
                min: current_min,
            })
        }
    }

    // decode
    option.callback.init(width,height);

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
    for x in 0..(height+dy-1)/dy {
        for y in 0..(width+dx-1)/dx {
            let mut yuv :Vec<Vec<u8>> = Vec::new();
            for scannummber in 0..mcu_size {
                let (dc_current,ac_current,i,tq) = scan[scannummber];
                let (zz,pred) = baseline_read(bitread
                            ,&dc_decode[dc_current].as_ref().unwrap()
                            ,&ac_decode[ac_current].as_ref().unwrap()
                            ,preds[i])?;
                preds[i] = pred;

                let sq = &super::util::ZIG_ZAG_SEQUENCE;
                let vals :Vec<i32> = (0..64).map(|i| 
                    zz[i] * quantization_tables[tq].q[i] as i32).collect();

                let uv :Vec<i32> = (0..64).map(|i| vals[sq[i]]).collect();
                let ff = idct(&uv);
                yuv.push(ff);
            }
            let data = if plane == 3 {yuv_to_rgb(&yuv,&component)} else {y_to_rgb(&yuv,&component)};
            option.callback.draw(x*dx,y*dy,dx,dy,&data);
        }
    }
    option.callback.terminate();
    Ok(None)
}
