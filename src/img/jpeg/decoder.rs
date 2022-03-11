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
    prev_rst: usize,
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
            prev_rst: 7,
            eof_flag: false,
        }
    }

    fn get_byte(self: &mut Self) -> Result<u8,ImgError> {
        if self.ptr >= self.buffer.len() {
            return Err(Simple(ErrorKind::OutboundIndex));
        }
        self.b = self.buffer[self.ptr];
        self.ptr = self.ptr + 1;
        Ok(self.b)
    }

    fn rst(self: &mut Self) -> Result<bool,ImgError> {
        if self.buffer[self.ptr] == 0xff {
            match self.buffer[self.ptr+1] {
                0xd0..=0xd7 =>  {    // RST    
                    self.ptr = self.ptr + 2;
                    self.bptr = 0;
                    return Ok(true);
                },
                _ => {
                    return Ok(false);
                },
            }
        }
        Ok(false)
    }

    fn next_marker(self: &mut Self) -> Result<u8,ImgError> {
        if self.get_byte()? != 0xff {
            return Err(SimpleAddMessage(ErrorKind::DecodeError,"Nothing marker".to_string()));
        }
        loop {
            let b = self.get_byte()?; 
            if b != 0xff {
                return Ok(b);
            }
        }
    }

    pub fn get_bit(self: &mut Self) -> Result<usize,ImgError> {
        if self.bptr == 0 {
            self.bptr = 8;
            if self.get_byte()? == 0xff {
                match self.get_byte()? {
                    0x00 => {
                        self.b = 0xff;
                    },
                    0xd0..=0xd7 =>  {    // RST    
                        let rst_no = (self.b & 0x7) as usize;
                        if rst_no != (self.prev_rst + 1) % 8 {
                            return Err(SimpleAddMessage(ErrorKind::DecodeError,format!("No Interval RST {} -> {}",self.prev_rst,rst_no)))
                        }

                        self.prev_rst = rst_no;
                        self.rst = true;
                        self.rst_ptr = self.ptr;
                    },
                    0xd9=> { // EOI
                        self.b = 0xff;
                    },
                    _ =>{
                        self.b = 0xff;
                        return Err(SimpleAddMessage(ErrorKind::DecodeError,"FF after  00 or RST".to_string()))
                    },                    
                }
            }
        }
        self.bptr -= 1;
        let r = (self.b  >> self.bptr) as usize & 0x1;
        Ok(r)
    }

    pub fn eof(self: &mut Self) -> bool {
        if self.buffer.len() - 2 < self.ptr || self.eof_flag
         {self.eof_flag=true}
        self.eof_flag
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
    let mut d = 0;
    let mut ll = 1;
    for l in 0..16 {
        d = (d << 1) | bit_reader.get_bit()?;
        if table.max[l] >= d as i32 {
            let p = d  - table.min[l] as usize + table.pos[l];
            return Ok(table.val[p] as u32)                      
        }
        ll = ll + 1;
    }
    Err(SimpleAddMessage(ErrorKind::OutboundIndex,"Huffman read Overflow".to_string()))  
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
    v = (v << 1) + bitread.get_bit()?;
  }
  Ok(v as i32)
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

/* fast idct is pre-calculate cos from fn idct
fn idct(f :&[i32]) -> Vec<u8> {
    let vals :Vec<u8> = (0..64).map(|i| {
        let (x,y) = ((i%8) as f32,(i/8) as f32);
        // IDCT from CCITT Rec. T.81 (1992 E) p.27 A3.3
        let mut val: f32=0.0;
        for u in 0..8 {
            let cu = if u == 0 {1.0 / 2.0_f32.sqrt()} else {1.0};
            for v in 0..8 {
                let cv = if v == 0 {1.0_f32 / 2.0_f32.sqrt()} else {1.0};
                val += cu * cv * (f[v*8 + u] as f32)
                    * ((2.0 * x + 1.0) * u as f32 * PI / 16.0_f32).cos()
                    * ((2.0 * y + 1.0) * v as f32 * PI / 16.0_f32).cos();
            }
        }
        val = val / 4.0;

        // level shift from CCITT Rec. T.81 (1992 E) p.26 A3.1
        let v = val.round() as i32 + 128;
        v.clamp(0,255) as u8
    }).collect();
    vals
}
*/

#[inline]
fn idct (f :&[i32]) -> Vec<u8> {
    let c_table :[[f32;8];8] = 
    [[ 0.70710678,  0.98078528,  0.92387953,  0.83146961,  0.70710678, 0.55557023,  0.38268343,  0.19509032],
    [ 0.70710678,  0.83146961,  0.38268343, -0.19509032, -0.70710678, -0.98078528, -0.92387953, -0.55557023],
    [ 0.70710678,  0.55557023, -0.38268343, -0.98078528, -0.70710678, 0.19509032,  0.92387953,  0.83146961],
    [ 0.70710678,  0.19509032, -0.92387953, -0.55557023,  0.70710678, 0.83146961, -0.38268343, -0.98078528],
    [ 0.70710678, -0.19509032, -0.92387953,  0.55557023,  0.70710678, -0.83146961, -0.38268343,  0.98078528],
    [ 0.70710678, -0.55557023, -0.38268343,  0.98078528, -0.70710678, -0.19509032,  0.92387953, -0.83146961],
    [ 0.70710678, -0.83146961,  0.38268343,  0.19509032, -0.70710678, 0.98078528, -0.92387953,  0.55557023],
    [ 0.70710678, -0.98078528,  0.92387953, -0.83146961,  0.70710678, -0.55557023,  0.38268343, -0.19509032]];
    let mut vals :Vec<u8> = (0..64).map(|_| 0).collect();
    for i in 0..16 {
        let (x,y) = ((i%4) as usize,(i/4) as usize);
        // IDCT from CCITT Rec. T.81 (1992 E) p.27 A3.3
        let mut val11 = 0.0;
        let mut val12 = 0.0;
        let mut val21 = 0.0;
        let mut val22 = 0.0;
        let mut plus_minus = 1.0;
        for u in 0..8 {
            let temp1 = f[0*8 + u] as f32 * c_table[y][0] + f[2*8 + u] as f32 * c_table[y][2]
                      + f[4*8 + u] as f32 * c_table[y][4] + f[6*8 + u] as f32 * c_table[y][6];

            let temp2 = f[1*8 + u] as f32 * c_table[y][0] + f[3*8 + u] as f32 * c_table[y][2]
                      + f[5*8 + u] as f32 * c_table[y][4] + f[7*8 + u] as f32 * c_table[y][6];

            let uval1 = temp1 + temp2;
            let uval2 = temp1 - temp2;
          
            val11 += uval1 * c_table[x][u];
            val12 += uval1 * c_table[x][u] * plus_minus;
            val21 += uval2 * c_table[x][u];
            val22 += uval2 * c_table[x][u] * plus_minus;
            plus_minus *= -1.0;
        }
        val11 /= 4.0;
        val12 /= 4.0;
        val21 /= 4.0;
        val22 /= 4.0;

        // level shift from CCITT Rec. T.81 (1992 E) p.26 A3.1
        let v = val11.round() as isize + 128 ;
        vals[y *8 + x] = v.clamp(0,255) as u8;
        let v = val12.round() as isize + 128 ;
        vals[y *8 + 7-x] = v.clamp(0,255) as u8;
        let v = val21.round() as isize + 128 ;
        vals[(7 - y) *8 + x] = v.clamp(0,255) as u8;
        let v = val22.round() as isize + 128 ;
        vals[(7 - y) *8 + 7-x] = v.clamp(0,255) as u8;
    }
    vals
}

/* fast_idct 
 * for application note 922 AP-922
 *   A Fast Precise Implementation of 8x8 Discrete Cosine
 *   Transform Using the Streaming SIMD Extensions and
 *   MMX™ Instructions
 *   Version 1.0
 *
 *   Copyright © Intel Corporation 1999
 *
 * base code from https://qiita.com/tobira-code/items/91f3578cd7ed5b19c1f9
 */
#[inline]
fn fast_idct (f :&[i32]) -> Vec<u8> {
    let g4 = 0.707106781186548 as f32;
    let g:[[f32;7];4]  = [
    /* row 0, 4 */
      [
        0.1733799806652680, /* g1 * 0.25 * g4 */
        0.1633203706095470, /* g2 * 0.25 * g4 */
        0.1469844503024200, /* g3 * 0.25 * g4 */
        0.1250000000000000, /* g4 * 0.25 * g4 */
        0.0982118697983878, /* g5 * 0.25 * g4 */
        0.0676495125182746, /* g6 * 0.25 * g4 */
        0.0344874224103679, /* g7 * 0.25 * g4 */
      ],
      /* row 1, 7 */
      [
        0.2404849415639110, /* g1 * 0.25 * g1 */
        0.2265318615882220, /* g2 * 0.25 * g1 */
        0.2038732892122290, /* g3 * 0.25 * g1 */
        0.1733799806652680, /* g4 * 0.25 * g1 */
        0.1362237766939550, /* g5 * 0.25 * g1 */
        0.0938325693794663, /* g6 * 0.25 * g1 */
        0.0478354290456362, /* g7 * 0.25 * g1 */
      ],
      /* row 2, 6 */
      [
        0.2265318615882220, /* g1 * 0.25 * g2 */
        0.2133883476483180, /* g2 * 0.25 * g2 */
        0.1920444391778540, /* g3 * 0.25 * g2 */
        0.1633203706095470, /* g4 * 0.25 * g2 */
        0.1283199917898340, /* g5 * 0.25 * g2 */
        0.0883883476483185, /* g6 * 0.25 * g2 */
        0.0450599888754343, /* g7 * 0.25 * g2 */
      ],
      /* row 3, 5 */
      [
        0.2038732892122290, /* g1 * 0.25 * g3 */
        0.1920444391778540, /* g2 * 0.25 * g3 */
        0.1728354290456360, /* g3 * 0.25 * g3 */
        0.1469844503024200, /* g4 * 0.25 * g3 */
        0.1154849415639110, /* g5 * 0.25 * g3 */
        0.0795474112858021, /* g6 * 0.25 * g3 */
        0.0405529186026822, /* g7 * 0.25 * g3 */
      ]];
    let t:[f32;3] = [
        0.414213562373095  /* t1 = g6/g2 */,
        0.198912367379658 /* t2 = g7/g1 */,
        0.668178637919299 /* t3 = g5/g3 */,
    ];
    
    let row2idx = [0,1,2,3,0,3,2,1];
    let mut _f = [[0_f32;8];8];
    let mut vals :Vec<u8> = (0..64).map(|_| 0).collect();

    for i in 0..8 {
        let idx = row2idx[i];
        /* P */
        let p = [
                f[0 +i] as f32,  /* 1 0 0 0 0 0 0 0 */
                f[2*8+i] as f32,  /* 0 0 1 0 0 0 0 0 */
                f[4*8+i] as f32,  /* 0 0 0 0 1 0 0 0 */
                f[6*8+i] as f32,  /* 0 0 0 0 0 0 1 0 */
                f[1*8+i] as f32,  /* 0 1 0 0 0 0 0 0 */
                f[3*8+i] as f32,  /* 0 0 0 1 0 0 0 0 */
                f[5*8+i] as f32,  /* 0 0 0 0 0 1 0 0 */
                f[7*8+i] as f32];  /* 0 0 0 0 0 0 0 1 */
        let tmp = [
            p[0] * g[idx][3],
            p[1] * g[idx][1],
            p[1] * g[idx][5],
            p[2] * g[idx][3],
            p[3] * g[idx][5],
            p[3] * g[idx][1]];
        /* M */
        let m = [
            tmp[0] + tmp[1] + tmp[3] + tmp[4],                                         /*  g4  g2  g4  g6  0  0  0  0 */
            tmp[0] + tmp[2] - tmp[3] - tmp[5],                                         /*  g4  g6 -g4 -g2  0  0  0  0 */
            tmp[0] - tmp[2] - tmp[3] + tmp[5],                                         /*  g4 -g6 -g4  g2  0  0  0  0 */
            tmp[0] - tmp[1] + tmp[3] - tmp[4],                                         /*  g4 -g2  g4 -g6  0  0  0  0 */
            p[4] * g[idx][0] + p[5] * g[idx][2] + p[6] * g[idx][4] + p[7] * g[idx][6], /*  0  0  0  0  g1  g3  g5  g7 */
            p[4] * g[idx][2] - p[5] * g[idx][6] - p[6] * g[idx][0] - p[7] * g[idx][4], /*  0  0  0  0  g3 -g7 -g1 -g5 */
            p[4] * g[idx][4] - p[5] * g[idx][0] + p[6] * g[idx][6] + p[7] * g[idx][2], /*  0  0  0  0  g5 -g1  g7  g3 */
            p[4] * g[idx][6] - p[5] * g[idx][4] + p[6] * g[idx][2] - p[7] * g[idx][0], /*  0  0  0  0  g7 -g5  g3 -g1 */
        ];
        /* A */
        _f[0][i]  = m[0] + m[4];  /*  1  0  0  0  1  0  0  0 */
        _f[1][i]  = m[1] + m[5];  /*  0  1  0  0  0  1  0  0 */
        _f[2][i]  = m[2] + m[6];  /*  0  0  1  0  0  0  1  0 */
        _f[3][i]  = m[3] + m[7];  /*  0  0  0  1  0  0  0  1 */
        _f[4][i]  = m[3] - m[7];  /*  0  0  0  1  0  0  0 -1 */
        _f[5][i]  = m[2] - m[6];  /*  0  0  1  0  0  0 -1  0 */
        _f[6][i]  = m[1] - m[5];  /*  0  1  0  0  0 -1  0  0 */
        _f[7][i]  = m[0] - m[4];  /*  1  0  0  0 -1  0  0  0 */
      }
    
      // column
      // add 8*26 = 208
      // mul 8*8  = 64
      // C = A F E B D P
      for i in 0..8 {
        /* P */
        let p = [
            _f[i][0],  /* 1 0 0 0 0 0 0 0 */
            _f[i][2],  /* 0 0 1 0 0 0 0 0 */
            _f[i][4],  /* 0 0 0 0 1 0 0 0 */
            _f[i][6],  /* 0 0 0 0 0 0 1 0 */
            _f[i][1],  /* 0 1 0 0 0 0 0 0 */
            _f[i][3],  /* 0 0 0 1 0 0 0 0 */
            _f[i][5],  /* 0 0 0 0 0 1 0 0 */
            _f[i][7],  /* 0 0 0 0 0 0 0 1 */
        ];
        /* D */
        /* g4  0  0  0  0  0  0  0 */
        /*  0  0 g4  0  0  0  0  0 */
        /*  0 g2  0  0  0  0  0  0 */
        /*  0  0  0 g2  0  0  0  0 */
        /*  0  0  0  0 g1  0  0  0 */
        /*  0  0  0  0  0  0  0 g1 */
        /*  0  0  0  0  0 g3  0  0 */
        /*  0  0  0  0  0  0 g3  0 */
        let d = [p[0],p[2],p[1],p[3], p[4],p[7],p[5],p[6]];
    
        /* B t1=g6/g2, t2=g7/g1, t3=g5/g3 */
        /*  1  1  0  0  0  0  0  0 */
        /*  1 -1  0  0  0  0  0  0 */
        /*  0  0  1 t1  0  0  0  0 */
        /*  0  0 t1 -1  0  0  0  0 */
        /*  0  0  0  0  1 t2  0  0 */
        /*  0  0  0  0 t2 -1  0  0 */
        /*  0  0  0  0  0  0  1 t3 */
        /*  0  0  0  0  0  0 t3 -1 */
        let b = [
                      d[0] +        d[1],
                      d[0] -        d[1],
                      d[2] + t[0] * d[3],
               t[0] * d[2] -        d[3],
                      d[4] + t[1] * d[5],
               t[1] * d[4] -        d[5],
                      d[6] + t[2] * d[7],
               t[2] * d[6] -        d[7],
        ];
    
        /* E */
        let e = [
            b[0] + b[2], /* 1  0  1  0  0  0  0  0 */
            b[1] + b[3], /* 0  1  0  1  0  0  0  0 */
            b[1] - b[3], /* 0  1  0 -1  0  0  0  0 */
            b[0] - b[2], /* 1  0 -1  0  0  0  0  0 */
            b[4] + b[6], /* 0  0  0  0  1  0  1  0 */
            b[4] - b[6], /* 0  0  0  0  1  0 -1  0 */
            b[5] + b[7], /* 0  0  0  0  0  1  0  1 */
            b[5] - b[7], /* 0  0  0  0  0  1  0 -1 */
        ];
        /* F g=g4*/
        let _f = [
            e[0],               /* 1  0  0  0  0  0  0  0 */
            e[1],               /* 0  1  0  0  0  0  0  0 */
            e[2],               /* 0  0  1  0  0  0  0  0 */
            e[3],               /* 0  0  0  1  0  0  0  0 */
            e[4],               /* 0  0  0  0  1  0  0  0 */
            g4 * (e[5] + e[6]), /* 0  0  0  0  0  g  g  0 */
            g4 * (e[5] - e[6]), /* 0  0  0  0  0  g -g  0 */
            e[7],               /* 0  0  0  0  0  0  0  1 */
        ];
        /* A */
        let v = (_f[0] + _f[4]).round()  as isize + 128;
        vals[i*8+0]  = v.clamp(0,255) as u8;    /* 1  0  0  0  1  0  0  0 */
        let v = (_f[1] + _f[5]).round()  as isize + 128;
        vals[i*8+1]  = v.clamp(0,255) as u8;    /* 0  1  0  0  0  1  0  0 */
        let v = (_f[2] + _f[6]).round()  as isize + 128;
        vals[i*8+2]  = v.clamp(0,255) as u8;    /* 0  0  1  0  0  0  1  0 */
        let v = (_f[3] + _f[7]).round()  as isize + 128;
        vals[i*8+3]  = v.clamp(0,255) as u8;    /* 0  0  0  1  0  0  0  1 */
        let v = (_f[3] - _f[7]).round()  as isize + 128;
        vals[i*8+4]  = v.clamp(0,255) as u8;    /* 0  0  0  1  0  0  0 -1 */
        let v = (_f[2] - _f[6]).round()  as isize + 128;
        vals[i*8+5]  = v.clamp(0,255) as u8;    /* 0  0  1  0  0  0 -1  0 */
        let v = (_f[1] - _f[5]).round()  as isize + 128;
        vals[i*8+6]  = v.clamp(0,255) as u8;    /* 0  1  0  0  0 -1  0  0 */
        let v = (_f[0] - _f[4]).round()  as isize + 128;
        vals[i*8+7]  = v.clamp(0,255) as u8;    /* 1  0  0  0 -1  0  0  0 */
    }

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
                    let shift = 4090;
                    let cy = gray[y * 8 + x] as i32;
                    let cb = yuv[u_map_cur][(((y + v * 8) / uy % 8) * 8)  + ((x + h * 8) / ux) % 8] as i32;
                    let cr = yuv[v_map_cur][(((y + v * 8) / vy % 8) * 8)  + ((x + h * 8) / vx) % 8] as i32;

                    let crr = (1.402 * shift as f32) as i32;
                    let cbg = (- 0.34414 * shift as f32) as i32;
                    let crg = (- 0.71414 * shift as f32) as i32;
                    let cbb = (1.772 * shift as f32) as i32;


                    let red  = cy + (crr * (cr - 128))/shift;
                    let green= cy + (cbg * (cb - 128) + crg * (cr - 128))/shift;
                    let blue = cy + (cbb * (cb - 128))/shift;

                    let red = if red > 255 {255} else if red < 0 {0} else {red as u8};
                    let green = if green > 255 {255} else if green < 0 {0} else {green as u8};
                    let blue = if blue > 255 {255} else if blue < 0 {0} else {blue as u8};

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
        // Make Huffman Table
    // Scan Header
    let header = JpegHaeder::new(buffer,0)?;

    if option.debug_flag > 0 {
        let boxstr = print_header(&header,option.debug_flag);
        (option.callback.verbose)(option.drawer,&boxstr)?;
    }
    
    if header.is_hierachical {
        return Err(SimpleAddMessage(ErrorKind::DecodeError,"Hierachical is not support".to_string()));
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

    if fh.is_huffman == false {
        return Err(SimpleAddMessage(ErrorKind::DecodeError,"This decoder suport huffman only".to_string()));
    }

    if fh.is_baseline == false {
        return Err(SimpleAddMessage(ErrorKind::DecodeError,"This Decoder support Baseline Only".to_string()));
    }

    if fh.is_differential == true {
        return Err(SimpleAddMessage(ErrorKind::DecodeError,"This Decoder not support differential".to_string()));
    }

    // decode
    (option.callback.init)(option.drawer,width,height)?;
    // take buffer for progressive 
    // progressive has 2mode
    //  - Spectral selection control
    //  - Successive approximation control
    /*  huffman for progressive
        EOBn -> 1 << n + get.bits(n)
        todo()
    */

    // loop start    

    let quantization_tables = header.quantization_tables.as_ref().unwrap();
    let (ac_decode,dc_decode) = huffman_extend(&header.huffman_tables.as_ref().unwrap());


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
            dy = usize::max(component[i].v * 8 ,dy);
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
                        return Ok(Some(WorningAddMessage(WorningKind::DataCorruption,r.fmt())));
                    }
                }
                preds[i] = pred;

                let sq = &super::util::ZIG_ZAG_SEQUENCE;
                let zz :Vec<i32> = (0..64).map(|i| 
                    zz[i] * quantization_tables[tq].q[i] as i32).collect();
                let zz :Vec<i32> = (0..64).map(|i| zz[sq[i]]).collect();
//                let ff = idct(&zz);
                let ff = fast_idct(&zz);
                yuv.push(ff);
            }

            let data = if plane == 3 {yuv_to_rgb(&yuv,&component)} else {y_to_rgb(&yuv,&component)};

            (option.callback.draw)(option.drawer,x*dx,y*dy,dx,dy,&data)?;

            if header.interval > 0 {
                mcu_interval = mcu_interval - 1;
                if mcu_interval == 0 { 
                    if  bitread.rst()? == true {
                        mcu_interval = header.interval as isize;
                        for i in 0..preds.len() {
                            preds[i] = 0;
                        }
                    } else {
                        worning = Some(WorningAddMessage(WorningKind::IlligalRSTMaker,"no mcu interval".to_string()));
                        return Ok(worning)
                    }
                } else if bitread.rst()? == true {
                    worning = Some(WorningAddMessage(WorningKind::IlligalRSTMaker,"mismatch mcu interval".to_string()));
                    mcu_interval = header.interval as isize;
                    for i in 0..preds.len() {
                        preds[i] = 0;
                    }
   //                 return Ok(worning);
                }
            }
        }
    }

    match bitread.next_marker() {
        Ok(marker) => {
            match marker {
                0xd9 => {   // EOI
                },
                0xdd => {
                    return Ok(Some(WorningAddMessage(WorningKind::UnexpectMaker,"DNL,No Support Multi scan/frame".to_string())))
                },
               _ => {
                    return Ok(Some(WorningAddMessage(WorningKind::UnexpectMaker,"No Support Multi scan/frame".to_string())))
                // offset = bitread.offset() -2
                // new_jpeg_header = read_makers(buffer[offset:],opt,false,true);
                // jpeg_header <= new Huffman Table if exit
                // jpeg_header <= new Quantize Table if exit
                // jpeg_header <= new Restart Interval if exit
                // jpeg_header <= new Add Comment Table if exit
                // jpeg_header <= new Add Appn if exit
                // goto loop
               },
            }
        },
        Err(..) => {
            worning = Some(WorningAddMessage(WorningKind::UnexpectMaker,"Not found EOI".to_string()));
        }
    }
    (option.callback.terminate)(option.drawer)?;
    Ok(worning)

}
