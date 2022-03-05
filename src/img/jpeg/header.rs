use crate::img::error::ImgError::Custom;
use crate::img::error::{ImgError,ErrorKind};
use crate::img::error::ImgError::{SimpleAddMessage};
use crate::img::util::{debug_print, debug_println};
use crate::img::io::*;

/* from SOS */
pub struct HuffmanScanHeader {
    pub ns: usize,
    pub csn: Vec<usize>,
    pub tdcn: Vec<usize>,
    pub tacn: Vec<usize>,
    pub ss: usize,
    pub se: usize,
    pub ah: usize,
    pub al: usize,
}

impl HuffmanScanHeader {
    pub fn new(ns: usize,csn: Vec<usize>,tdcn: Vec<usize>,tacn: Vec<usize>,ss:usize, se:usize,ah: usize,al :usize) -> Self{
        Self {
            ns,
            csn,
            tdcn,
            tacn,
            ss,
            se,
            ah,
            al,
        }
    }
}


/* from DHT */
pub struct HuffmanTable {
    pub ac: bool,
    pub no: usize,
    pub len: Vec<usize>,
    pub pos: Vec<usize>,
    pub val: Vec<usize>,
}

impl HuffmanTable {
    pub fn new(ac:bool,no:usize,len: Vec<usize>,pos: Vec<usize>,val: Vec<usize>) -> Self {
        Self {
            ac,
            no,
            len,
            pos,
            val,
        }
    }
}

/* from DQT */
pub struct QuantizationTable {
    pub presision: usize,
    pub no: usize,
    pub q: Vec<usize>,
}

impl QuantizationTable {
    pub fn new(presision:usize,no: usize,q: Vec<usize>) -> Self {
        Self {
            presision,
            no,
            q,
        }
    }

}
/* SOF */
pub struct Component{
    pub c: usize,
    pub h: usize,
    pub v: usize,
    pub tq: usize
}

pub struct FrameHeader {
    pub baseline: bool,
    pub sequantial: bool,
    pub progressive: bool,
    pub lossress: bool,
    pub differential: bool,
    pub huffman: bool,
    pub width: usize,
    pub height: usize,
    pub bitperpixel: usize,
    pub plane: usize,
    pub component: Option<Vec<Component>>,
}

impl FrameHeader {
    #[warn(unused_assignments)]
    pub fn new(num: usize,buffer: &[u8]) -> Self {
        let mut baseline: bool = false;
        let mut sequantial: bool = false;
        let mut progressive: bool = false;
        let mut lossress: bool = false;
        let mut differential: bool = false;
        let huffman;
        let width: usize;
        let height: usize;
        let bitperpixel: usize;
        let plane: usize;
        let mut component: Vec<Component>;

        if num & 0x03 == 0x00 {
            baseline = true;
            debug_print!("Baseline ");
        }
        if num & 0x03 == 0x01 {
            sequantial = true;
            debug_print!("Sequential ");
        }
        if num & 0x03 == 0x02 
        {
            progressive = true;
            debug_print!("Progressive ");
        }
        if num & 0x03 == 0x03 {
            lossress = true;
            debug_print!("Lossress ");
        }
        if num & 0x08 == 0x00 {
            huffman = true;
            debug_print!("Huffman ");
        } else {
            huffman = false;
            debug_print!("Arithmetic coding ");
        }
        if num & 0x04 == 0x00 {
            differential = false;
            debug_print!("non differential");
        }
        if num & 0x04 == 0x04 {
            differential = true;
            debug_print!("differential");
        }
        debug_println!("");

        let p = read_byte(&buffer,0) as i32;
        bitperpixel = p as usize;
        height = read_u16be(&buffer,1) as usize;
        width = read_u16be(&buffer,3) as usize;
        let nf = read_byte(&buffer,5) as i32;
        plane = nf as usize;

        let mut ptr = 6;

        component = Vec::new();

        for _ in 0..nf {
            let c = read_byte(&buffer,ptr) as usize;
            let h = (read_byte(&buffer,ptr + 1) >> 4) as usize;
            let v = (read_byte(&buffer,ptr + 1) & 0x07) as usize;
            let tq = read_byte(&buffer,ptr + 2) as usize;
            ptr = ptr + 3;
//            debug_println!("No{} {}x{} Table{}", c,h,v,tq);
            component.push(Component{c,h,v,tq});
        }
 
        Self {
            baseline,
            sequantial,
            progressive,
            lossress,
            differential,
            huffman,
            width,
            height,
            bitperpixel,
            plane,
            component: Some(component), 
        }
    }
}


/* APP0 */
pub struct Jfif {
    pub version: u16,
    pub resolusion_unit: usize,
    pub x_resolusion: usize,
    pub y_resolusion: usize,
    pub width: usize,
    pub height: usize,
    pub thumnail: Option<Vec<u8>>,  // (width*height*3)  + tag
}

#[allow(unused)]
pub struct Jfxx {
    pub id : String,// +2   // JFXX\0
    pub ver: usize, // +7
    pub t: usize,   // +9   
    pub width: usize,   //+10 if t == 11 or 12
    pub height: usize,  //+11 if t == 11 or 12
    pub palette: Option<Vec<(u8,u8,u8)>>, // if t ==11
    pub thumnail: Option<Vec<u8>>,  // +16 - (xt*yt*3)
}

#[allow(unused)]
pub struct AdobeApp14 {
    pub dct_encode_version: usize,
    pub flag1: usize,
    pub flag2: usize,
    pub color_transform: usize,
}

pub type Exif = crate::img::tiff::header::TiffHeaders;

#[allow(unused)]
pub struct Ducky {
    pub quality: usize,
    pub comment: String,
    pub copyright: String,
}

#[allow(unused)]
pub struct UnknownApp {
    pub number : usize,
    pub tag : String,
    pub lenghth : usize,
}


pub struct JpegHaeder {
    pub width : usize,
    pub height: usize,
    pub bpp: usize,
    pub frame_header:Option<FrameHeader>,
    pub huffman_tables:Option<Vec<HuffmanTable>>,
    pub huffman_scan_header:Option<HuffmanScanHeader>,
    pub quantization_tables:Option<Vec<QuantizationTable>>,
    pub line: usize,
    pub interval :usize,
    pub imageoffset: usize,
    pub comment: Option<String>,
    pub jpeg_app_headers: Option<Vec<JpegAppHeaders>>,
}

#[allow(unused)]
pub enum JpegAppHeaders {
    Jfif(Jfif),
    Exif(Exif),
    Ducky(Ducky),
    Adobe(AdobeApp14),
    Unknown(UnknownApp),
}

fn read_app(num: usize,tag :&String,buffer :&[u8],mut ptr :usize,mut len :usize) -> Result<JpegAppHeaders,ImgError> {
    match num {
        0 => {
            match tag.as_str() {
                "JFIF" => {
                    let version = read_u16be(&buffer,ptr) as u16;
                    let unit = read_byte(&buffer,ptr + 2) as usize;
                    let xr = read_u16be(&buffer,ptr + 3) as usize;
                    let yr = read_u16be(&buffer,ptr + 5) as usize;
                    let width = read_byte(&buffer,ptr + 7) as usize;
                    let height = read_byte(&buffer,ptr + 8) as usize;


                    debug_print!("Version: {:>04X} ",version);
                    debug_print!("ResolutionUnit: {} ",if unit == 1 {" inches"} else if unit == 2 {"cm"} else {"None"} );
                    debug_println!("Resolution X{} Y{}",xr,yr);
                    debug_println!("Thumbnail {}x{}",width,height);
                    let jfif :Jfif  = Jfif{
                        version: version,
                        resolusion_unit: unit,
                        x_resolusion: xr,
                        y_resolusion: yr,
                        width: width,
                        height: height,
                        thumnail: None,  // (width*height*3)  + tag
                    };

                    return Ok(JpegAppHeaders::Jfif(jfif))
                },
                _ => {

                }
            }
        },
        1 => {
            match tag.as_str() {
                "Exif" => {
                    ptr = ptr + 1; // start 6byte
                    len = len - 1;
                    let buf :Vec<u8> = (0..len)
                        .map(|i| {buffer[ptr + i]})
                        .collect();

                    super::super::tiff::header::read_tags(&buf)?;
                },
                _ => {
                }
            }
        },
        12 => {
            match tag.as_str() {
                "Ducky" => {
                    let q = read_u32be(&buffer,ptr) as usize;
                    ptr = ptr + 4;
                    len = len - 4;
                    let comment = read_string(&buffer,ptr,len);
                    ptr = ptr + comment.len() + 1;
                    len = len - comment.len() + 1;
                    let copyright = read_string(&buffer,ptr,len);
                    debug_println!("Quality: {:>08X}",q);
                    debug_println!("{} {}",comment,copyright);
                    return Ok(JpegAppHeaders::Ducky(Ducky{quality: q,comment: comment,copyright: copyright}));
                },
                _ => {
                },
            }
        },
        14 => {
            match tag.as_str() {
                "Adobe" => {
                    let ver = read_byte(&buffer, ptr) as usize;
                    let flag1 = read_byte(&buffer, ptr + 1) as usize;
                    let flag2 = read_byte(&buffer, ptr + 2) as usize;
                    let ct = read_byte(&buffer, ptr + 3) as usize;
                    debug_println!("DCTEncodeVersion:{} Flag1:{} Flag2:{} ColorTransform {}"
                        ,ver,flag1,flag2,if ct == 1 {"YCbCr"} else if ct ==2 {"YCCK"} else {"Unknown"} );
                    return Ok(JpegAppHeaders::Adobe(AdobeApp14{dct_encode_version: ver,flag1 :flag1,flag2: flag2,color_transform: ct}));
                },
                _ => {
                }
            }
        },
        _ => {
        }
    }
    Ok(JpegAppHeaders::Unknown(UnknownApp{number:num ,tag: tag.to_string(),lenghth: len}))
}

impl JpegHaeder {
    pub fn new(buffer :&[u8],opt :usize) -> Result<Self,ImgError> {
        let flag = opt;
        let mut _flag = false;
        let mut _dqt_flag = false;
        let mut _dht_flag = false;
        let mut _sof_flag = false;
        let mut _sos_flag = false;
        let mut width : usize = 0;
        let mut height: usize = 0;
        let mut bpp: usize = 0;
        let mut _huffman_tables:Vec<HuffmanTable> = Vec::new();
        let huffman_tables:Option<Vec<HuffmanTable>>;
        let mut huffman_scan_header:Option<HuffmanScanHeader> = None;
        let mut quantization_tables:Vec<QuantizationTable> = Vec::new();
        let mut line: usize = 0;
        let mut interval :usize = 0;
        let mut frame_header:Option<FrameHeader> = None;
        let mut comment: Option<String> = None;
        let mut _jpeg_app_headers: Vec<JpegAppHeaders> = Vec::new();
        let jpeg_app_headers: Option<Vec<JpegAppHeaders>>;
        let mut offset = 0;

        while offset < 16 {
            let soi = read_u16be(buffer,offset);
//            debug_println!("{:>04x}",soi);
            if soi == 0xffd8 {break};
            offset = offset + 1;
        }

        if offset >= 16 {
            return Err(Custom("Not Jpeg".to_string()))
        }

        while offset < buffer.len() {
            let byte = buffer[offset];  // read byte
            if byte == 0xff { // header head
                let nextbyte :u8 = read_byte(&buffer,offset + 1);
                offset = offset + 2;

                match nextbyte {
                    0xc4 => { // DHT maker
                        _dht_flag = true;
                        let length: usize = (buffer[offset] as usize) << 8 | buffer[offset + 1] as usize;

                        let mut size :usize = 2;
                        while size < length {
                            let tc = read_byte(&buffer,offset + 2) >> 4;
                            let th = read_byte(&buffer,offset + 2) & 0x0f;
                            let ac = if tc == 0 { false } else { true };

                            let no = th as usize;
                            size = size + 1;
                            let mut pss :usize = 0;
                            let mut len :Vec<usize> = Vec::with_capacity(16);
                            let mut p :Vec<usize> = Vec::with_capacity(16);
                            let mut val :Vec<usize> = Vec::new();
                            for i in 0..16 {
                                let l = read_byte(&buffer,offset + 3 + i) as usize;

                                p.push(pss);
                                len.push(l);
                                for _ in 0..l {
                                    val.push(read_byte(&buffer,offset + 19 + pss) as usize);

                                    pss =  pss + 1;
                                }

                                size = size + 1;
                            }

                            size = size + pss;
                            _huffman_tables.push(HuffmanTable::new(ac,no,len,p,val));
                        }

                        offset = offset + length; // skip
                    },
                    0xc0..=0xcf => {  // SOF Frame Headers;
                        if !_sof_flag {
                            _sof_flag = true;
                            let num = (nextbyte & 0x0f) as usize;
                            if flag & 0x04 != 0 {
                                debug_print!("\nSOF{} ",num);
                            }
                            let length = read_u16be(&buffer,offset) as usize;
                            if flag & 0x04 != 0 {
                                debug_print!(" {} ",length);
                            }
                            let buf = read_bytes(&buffer,offset + 2,length - 2);
                            let fh = FrameHeader::new(num,&buf);
                            debug_println!("{}x{} pixel - {}bit color ",fh.width,fh.height,fh.bitperpixel * fh.plane);
                            width = fh.width;
                            height = fh.height;
                            bpp = fh.bitperpixel * fh.plane;
                            if flag & 0x04 != 0 {
                                debug_println!(" {}byte",length);
                            }
                            frame_header = Some(fh);
                            offset = offset + length; //skip
                        }
                    },
                    0xd8 => { // Start of Image
                        _flag = true;
                    },
                    0xd9=> { // End of Image
                        return Err(SimpleAddMessage(ErrorKind::DecodeError ,"Unexpect EOI".to_string()));
                    },
                    0xda=> { // SOS Scan header
                        _sos_flag = true;
                        let length: usize = read_u16be(&buffer,offset) as usize;
                        if flag & 0x04 == 0x04 {
                            debug_println!("\nHuffman Scan Header {}byte",length);
                        }
                        let mut ptr = offset + 2;
                        let ns = read_byte(&buffer,ptr) as usize;
                        if flag & 0x04 == 0x04 {
                            debug_println!("ns {}",ns);
                        }
                        ptr = ptr + 1;
                        let mut csn: Vec<usize> = Vec::with_capacity(ns);
                        let mut tdn: Vec<usize> = Vec::with_capacity(ns);
                        let mut tan: Vec<usize> = Vec::with_capacity(ns);
                        for i in 0..ns {
                            csn.push(read_byte(&buffer,ptr) as usize);
                            tdn.push((read_byte(&buffer,ptr + 1) >> 4) as usize);
                            tan.push((read_byte(&buffer,ptr + 1) & 0xf ) as usize);
                            ptr = ptr + 2;
                            if flag & 0x04 == 0x04 {
                                debug_println!("ID {} Tabel DC {} Tabel AC {}",csn[i],tdn[i],tan[i]);
                            }
                        }
                        let ss = read_byte(&buffer,ptr) as usize;
                        let se = read_byte(&buffer,ptr) as usize;
                        let ah = (read_byte(&buffer,ptr + 2) >> 4) as usize;
                        let al = (read_byte(&buffer,ptr + 2) & 0xf ) as usize;
                        if flag & 0x04 == 0x04 {
                            debug_println!("Start {} End {} Shift Before {} Shift After{}",ss,se,ah,al);
                        }
                        huffman_scan_header = Some(HuffmanScanHeader::new(ns,csn,tdn,tan,ss,se,ah,al));

                        offset = offset + length; //skip
                        break; // next is imagedata
                    },
                    0xdb =>{ // Define Quantization Table
                        _dqt_flag = true;
                        let length: usize = read_u16be(&buffer,offset) as usize;
                        debug_println!("\nQuantization Table {}byte",length);
                        // read_dqt;
                        let mut pos :usize = 2;
                        while pos < length {
                            let mut quantizations :Vec<usize> = Vec::with_capacity(64);
                            let presision :usize;
                            let p = read_byte(&buffer,pos + offset) >> 4;
                            let no = (read_byte(&buffer,pos + offset) & 0x0f) as usize;
                            debug_print!("#{} ",no);
                            pos = pos + 1;
                            if p == 0 {
                                presision = 8;
                                debug_print!("{}bit ",presision);
                                for _ in 0..64 {
                                    quantizations.push(read_byte(&buffer,pos + offset) as usize);
                                    debug_print!("{} ",read_byte(&buffer,pos + offset));
                                    pos = pos + 1;
                                }
                            } else {
                                presision = 16;
                                debug_print!("{}bit ",presision);
                                for _ in 0..64 {
                                    quantizations.push(read_u16be(&buffer,pos + offset) as usize);
                                    debug_print!("{} ",read_u16be(&buffer,pos + offset));
                                    pos = pos + 2;
                                }
                            }
                            if flag & 0x04 == 0x04 {
                                debug_println!(" - {}bytes ",pos);
                            }
                            quantization_tables.push(QuantizationTable::new(presision,no,quantizations));
                        }
                        offset = offset + length; // skip
                    },
                    0xdc =>{ // DNL Define Number Lines
                        _dqt_flag = true;
                        let length: usize = read_u16be(&buffer,offset) as usize;
                        let nl = read_u16be(&buffer,offset) as usize;
                        line = nl;
                        if flag & 0x04 == 0x04 {
                            debug_println!("\nDNL {}",nl);
                        }
                        // read_dqt;
                        offset = offset + length; // skip
                    },
                    0xdd => { // Define Restart Interval
                        if flag & 0x04 == 0x04 {
                            debug_println!("\nDefine Restart Interval");
                        }
                        let length = read_u16be(&buffer,offset) as usize;
                        let ri = read_u16be(&buffer,offset + 2);
                        interval = ri as usize;
                        if flag & 0x04 == 0x04 {
                            debug_println!("Resrart Interval {}",ri);
                        }
                        offset = offset + length; // skip
                    },
                    0xfe => { // Comment
                        if flag & 0x01 == 0x01 {    
                            debug_println!("\nComment");
                        }
                        let length = read_u16be(&buffer,offset) as usize;
                        comment = Some(String::from_utf8(buffer[offset+2..offset+length].to_vec()).unwrap());
                        if flag & 0x01 == 0x01 {    
                            debug_println!("{}", comment.as_ref().unwrap());
                        }
                        offset = offset + length; // skip
                    },
                    0xe0..=0xef => { // Applications 
                        let num = (nextbyte & 0xf) as usize;
                        let length = read_u16be(&buffer,offset) as usize;
                        let tag = read_string(buffer,offset + 2,length -2);
                        let len = length - 2 - tag.len() + 1;
                        let ptr = 2 + tag.len() + 1 + offset;
//                        if flag & 0x02 == 0x02 {
                            debug_println!("\nAPP{} {} {}bytes {} {}",num,tag,length,ptr,len);
                            let result = read_app(num , &tag, &buffer, ptr, len)?;
                            _jpeg_app_headers.push(result);
//                        }
                        offset = offset + length; // skip
                    },
                    0xff => { // padding
                        offset = offset + 1;
                    },
                    0x00 => { //data
                        // skip
                    },
                    0xd0..=0xd7 => {   // REST0-7
//                        debug_println!("\nRSET {}",nextbyte & 0x7);                
                    },
                    _ => {
                        debug_println!("unimpriment marker {:>02X}",nextbyte);
                        let length = read_u16be(&buffer,offset) as usize;
                        offset = offset + length;
                    }
                }
            } else {
//                debug_print!("{:02x} ",byte);
//                return Err("Not Jpeg".to_string());
                offset = offset +1;
            }

        }

        if _sof_flag && _sos_flag && _dht_flag && _dqt_flag == false {
            return Err(SimpleAddMessage(ErrorKind::IlligalData,"Maker is shortage".to_string()));
        }

        if _jpeg_app_headers.len() > 0 {
            jpeg_app_headers = Some(_jpeg_app_headers);
        } else {
            jpeg_app_headers = None;
        }

        if _huffman_tables.len() > 0 {
            huffman_tables = Some(_huffman_tables);
        } else {
            huffman_tables = None;
        }

        Ok(Self {
            width,
            height,
            bpp,
            frame_header,
            huffman_scan_header,
            huffman_tables,
            quantization_tables: Some(quantization_tables),
            line,
            interval,
            imageoffset:  offset,
            comment,
            jpeg_app_headers,
        })
    }
}