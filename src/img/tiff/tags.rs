use super::super::io::read_string;
use super::super::util::*;
use super::header::DataPack;
use super::header::print_data;

pub fn gps_mapper(tag :u16, data: &DataPack) {
    match tag {
        0x0000 => {
            debug_println!("GPSVersionID");
            print_data(&data);
           
           
           },
                   0x0001 => {
            debug_println!("GPSLatitudeRef");
            print_data(&data);
           
           
           },
           
                   0x0002 => {
            debug_println!("GPSLatitude");
            print_data(&data);
           
           
           },
                   0x0003 => {
            debug_println!("GPSLongitudeRef");
            print_data(&data);
           
           
           },
                       
                   0x0004 => {
            debug_println!("GPSLongitude");
            print_data(&data);
           
           
           },
                   0x0005 => {
            debug_println!("GPSAltitudeRef");
            print_data(&data);
           
           
           },
                       
                   0x0006 => {
            debug_println!("GPSAltitude");
            print_data(&data);
           
           
           },
                   0x0007 => {
            debug_println!("GPSTimeStamp");
            print_data(&data);
           
           
           },
                   0x0008 => {
            debug_println!("GPSSatellites");
            print_data(&data);
           
           
           },
                   0x0009 => {
            debug_println!("GPSStatus");
            print_data(&data);
           
           
           },
                       
                   0x000a => {
            debug_println!("GPSMeasureMode");
            print_data(&data);
           
           
           },
                       
                   0x000b => {
            debug_println!("GPSDOP");
            print_data(&data);
           
           
           },
                   0x000c => {
            debug_println!("GPSSpeedRef");
            print_data(&data);
           
           
           },
                       
                   0x000d => {
            debug_println!("GPSSpeed");
            print_data(&data);
           
           
           },
                   0x000e => {
            debug_println!("GPSTrackRef");
            print_data(&data);
           
           
           },
                       
                   0x000f => {
            debug_println!("GPSTrack");
            print_data(&data);
           
           
           },
                   0x0010 => {
            debug_println!("GPSImgDirectionRef");
            print_data(&data);
           
           
           },
                       
                   0x0011 => {
            debug_println!("GPSImgDirection");
            print_data(&data);
           
           
           },
                   0x0012 => {
            debug_println!("GPSMapDatum");
            print_data(&data);
           
           
           },
                   0x0013 => {
            debug_println!("GPSDestLatitudeRef");
            print_data(&data);
           
           
           },
                   0x0014 => {
            debug_println!("GPSDestLatitude");
            print_data(&data);
           
           
           },
                   0x0015 => {
            debug_println!("GPSDestLongitudeRef");
            print_data(&data);
           
           
           },
                   0x0016 => {
            debug_println!("GPSDestLongitude");
            print_data(&data);
           
           
           },
                   0x0017 => {
            debug_println!("GPSDestBearingRef");
            print_data(&data);
           
           
           },
                   0x0018 => {
            debug_println!("GPSDestBearing");
            print_data(&data);
           
           
           },
                   0x0019 => {
            debug_println!("GPSDestDistanceRef");
            print_data(&data);
           
           
           },
                   0x001a => {
            debug_println!("GPSDestDistance");
            print_data(&data);
           
           
           },
                   0x001b => {
            debug_println!("GPSProcessingMethod");
            print_data(&data);
           
           
           },
                   0x001c => {
            debug_println!("GPSAreaInformation");
            print_data(&data);
           
           
           },
                   0x001d => {
            debug_println!("GPSDateStamp");
            print_data(&data);
           
           
           },
                   0x001e => {
            debug_println!("GPSDifferential");
            print_data(&data);
           
           
           },
                   0x001f => {
            debug_println!("GPSHPositioningError");
            print_data(&data);
           
           
           },
         _=> {},
        }  
}


pub fn tag_mapper(tag :u16, data: &DataPack) {
   match tag {
    0x0001 => {
        debug_print!("InteropIndex ");
        print_data(&data);
     },
    0x0002 => {
     debug_print!("InteropVersion ");

         print_data(&data)
     },
    0x000b => {
     debug_print!("ProcessingSoftware ");

         print_data(&data)

     
     },
    0x00fe => {
     debug_print!("SubfileType ");

         print_data(&data)

     
     },
    0x00ff => {
     debug_print!("OldSubfileType ");

         print_data(&data)

     
     },
    0x0100 => {
     debug_print!("ImageWidth ");

         print_data(&data)

     
     },
    0x0101 => {
     debug_print!("ImageHeight ");

         print_data(&data)

     
     },
    0x0102 => {
     debug_print!("BitsPerSample ");

         print_data(&data)

     
     },
    0x0103 => {
     debug_print!("Compression ");

         print_data(&data)

     
     },
    0x0106 => {
     debug_print!("PhotometricInterpretation ");

         print_data(&data)

     
     },
    0x0107 => {
     debug_print!("Thresholding ");

         print_data(&data)

     
     },
    0x0108 => {
     debug_print!("CellWidth ");

         print_data(&data)

     
     },
    0x0109 => {
     debug_print!("CellLength ");

         print_data(&data)

     
     },
    0x010a => {
     debug_print!("FillOrder ");

         print_data(&data)

     
     },
    0x010d => {
     debug_print!("DocumentName ");

         print_data(&data)

     
     },
    0x010e => {
     debug_print!("ImageDescription ");

         print_data(&data)

     
     },
    0x010f => {
     debug_print!("Make ");

         print_data(&data)

     
     },
    0x0110 => {
     debug_print!("Model ");

         print_data(&data)

     
     },
    0x0111 => {
     debug_print!("StripOffsets ");

         print_data(&data)

     
     },
    0x0112 => {
     debug_print!("Orientation ");

         print_data(&data)

     
     },
    0x0115 => {
     debug_print!("SamplesPerPixel ");

         print_data(&data)

     
     },
    0x0116 => {
     debug_print!("RowsPerStrip ");

         print_data(&data)

     
     },
    0x0117 => {
     debug_print!("StripByteCounts ");

         print_data(&data)

     
     },
    0x0118 => {
     debug_print!("MinSampleValue ");

         print_data(&data)

     
     },
    0x0119 => {
     debug_print!("MaxSampleValue ");

         print_data(&data)

     
     },
    0x011a => {
     debug_print!("XResolution ");

         print_data(&data)

     
     },
    0x011b => {
     debug_print!("YResolution ");

         print_data(&data)

     
     },
    0x011c => {
     debug_print!("PlanarConfiguration ");

         print_data(&data)

     
     },
    0x011d => {
     debug_print!("PageName ");

         print_data(&data)

     
     },
    0x011e => {
     debug_print!("XPosition ");

         print_data(&data)

     
     },
    0x011f => {
     debug_print!("YPosition ");

         print_data(&data)

     
     },
    0x0120 => {
     debug_print!("FreeOffsets ");

         print_data(&data)

     
     },
    0x0121 => {
     debug_print!("FreeByteCounts ");

         print_data(&data)

     
     },
    0x0122 => {
     debug_print!("GrayResponseUnit ");

         print_data(&data)

     
     },
    0x0123 => {
     debug_print!("GrayResponseCurve ");

         print_data(&data)

     
     },
    0x0124 => {
     debug_print!("T4Options ");

         print_data(&data)

     
     },
    0x0125 => {
     debug_print!("T6Options ");

         print_data(&data)

     
     },
    0x0128 => {
     debug_print!("ResolutionUnit ");

         print_data(&data)

     
     },
    0x0129 => {
     debug_print!("PageNumber ");

         print_data(&data)

     
     },
    0x012c => {
     debug_print!("ColorResponseUnit ");

         print_data(&data)

     
     },
    0x012d => {
     debug_print!("TransferFunction ");

         print_data(&data)

     
     },
    0x0131 => {
     debug_print!("Software ");

         print_data(&data)

     
     },
    0x0132 => {
     debug_print!("ModifyDate ");

         print_data(&data)

     
     },
    0x013b => {
     debug_print!("Artist ");

         print_data(&data)

     
     },
    0x013c => {
     debug_print!("HostComputer ");

         print_data(&data)

     
     },
    0x013d => {
     debug_print!("Predictor ");

         print_data(&data)

     
     },
    0x013e => {
     debug_print!("WhitePoint ");

         print_data(&data)

     
     },
    0x013f => {
     debug_print!("PrimaryChromaticities ");

         print_data(&data)

     
     },
    0x0140 => {
     debug_print!("ColorMap ");

         print_data(&data)

     
     },
    0x0141 => {
     debug_print!("HalftoneHints ");

         print_data(&data)

     
     },
    0x0142 => {
     debug_print!("TileWidth ");

         print_data(&data)

     
     },
    0x0143 => {
     debug_print!("TileLength ");

         print_data(&data)

     
     },
    0x0144 => {
     debug_print!("TileOffsets ");

         print_data(&data)

     
     },
    0x0145 => {
     debug_print!("TileByteCounts ");

         print_data(&data)

     
     },
    0x0146 => {
     debug_print!("BadFaxLines ");

         print_data(&data)

     
     },
    0x0147 => {
     debug_print!("CleanFaxData ");

         print_data(&data)

     
     },
    0x0148 => {
     debug_print!("ConsecutiveBadFaxLines ");

         print_data(&data)

     
     },
    0x014a => {
     debug_print!("SubIFD ");

         print_data(&data)

     
     },
    0x014c => {
     debug_print!("InkSet ");

         print_data(&data)

     
     },
    0x014d => {
     debug_print!("InkNames ");

         print_data(&data)

     
     },
    0x014e => {
     debug_print!("NumberofInks ");

         print_data(&data)

     
     },
    0x0150 => {
     debug_print!("DotRange ");

         print_data(&data)

     
     },
    0x0151 => {
     debug_print!("TargetPrinter ");

         print_data(&data)

     
     },
    0x0152 => {
     debug_print!("ExtraSamples ");

         print_data(&data)

     
     },
    0x0153 => {
     debug_print!("SampleFormat ");

         print_data(&data)

     
     },
    0x0154 => {
     debug_print!("SMinSampleValue ");

         print_data(&data)

     
     },
    0x0155 => {
     debug_print!("SMaxSampleValue ");

         print_data(&data)

     
     },
    0x0156 => {
     debug_print!("TransferRange ");

         print_data(&data)

     
     },
    0x0157 => {
     debug_print!("ClipPath ");

         print_data(&data)

     
     },
    0x0158 => {
     debug_print!("XClipPathUnits ");

         print_data(&data)

     
     },
    0x0159 => {
     debug_print!("YClipPathUnits ");

         print_data(&data)

     
     },
    0x015a => {
     debug_print!("Indexed ");

         print_data(&data)

     
     },
    0x015b => {
     debug_print!("JPEGTables ");

         print_data(&data)

     
     },
    0x015f => {
     debug_print!("OPIProxy ");

         print_data(&data)

     
     },
    0x0190 => {
     debug_print!("GlobalParametersIFD ");

         print_data(&data)

     
     },
    0x0191 => {
     debug_print!("ProfileType ");

         print_data(&data)

     
     },
    0x0192 => {
     debug_print!("FaxProfile ");

         print_data(&data)

     
     },
    0x0193 => {
     debug_print!("CodingMethods ");

         print_data(&data)

     
     },
    0x0194 => {
     debug_print!("VersionYear ");

         print_data(&data)

     
     },
    0x0195 => {
     debug_print!("ModeNumber ");

         print_data(&data)

     
     },
    0x01b1 => {
     debug_print!("Decode ");

         print_data(&data)

     
     },
    0x01b2 => {
     debug_print!("DefaultImageColor ");

         print_data(&data)

     
     },
    0x01b3 => {
     debug_print!("T82Options ");

         print_data(&data)

     
     },
    0x01b5 => {
     debug_print!("JPEGTables ");

         print_data(&data)

     
     },
    0x0200 => {
     debug_print!("JPEGProc ");

         print_data(&data)

     
     },
    0x0201 => {
     debug_print!("ThumbnailOffset ");

         print_data(&data)

     
     },
    0x0202 => {
     debug_print!("ThumbnailLength ");

         print_data(&data)

     
     },
    0x0203 => {
     debug_print!("JPEGRestartInterval ");

         print_data(&data)

     
     },
    0x0205 => {
     debug_print!("JPEGLosslessPredictors ");

         print_data(&data)

     
     },
    0x0206 => {
     debug_print!("JPEGPointTransforms ");

         print_data(&data)

     
     },
    0x0207 => {
     debug_print!("JPEGQTables ");

         print_data(&data)

     
     },
    0x0208 => {
     debug_print!("JPEGDCTables ");

         print_data(&data)

     
     },
    0x0209 => {
     debug_print!("JPEGACTables ");

         print_data(&data)

     
     },
    0x0211 => {
     debug_print!("YCbCrCoefficients ");

         print_data(&data)

     
     },
    0x0212 => {
     debug_print!("YCbCrSubSampling ");

         print_data(&data)

     
     },
    0x0213 => {
     debug_print!("YCbCrPositioning ");

         print_data(&data)

     
     },
    0x0214 => {
     debug_print!("ReferenceBlackWhite ");

         print_data(&data)

     
     },
    0x022f => {
     debug_print!("StripRowCounts ");

         print_data(&data)

     
     },
    0x02bc => {
     debug_print!("ApplicationNotes ");

         print_data(&data)

     
     },
    0x03e7 => {
     debug_print!("USPTOMiscellaneous ");

         print_data(&data)

     
     },
    0x1000 => {
     debug_print!("RelatedImageFileFormat ");

         print_data(&data)

     
     },
    0x1001 => {
     debug_print!("RelatedImageWidth ");

         print_data(&data)

     
     },
    0x1002 => {
     debug_print!("RelatedImageHeight ");

         print_data(&data)

     
     },
    0x4746 => {
     debug_print!("Rating ");

         print_data(&data)

     
     },
    0x4747 => {
     debug_print!("XP_DIP_XML ");

         print_data(&data)

     
     },
    0x4748 => {
     debug_print!("StitchInfo ");

         print_data(&data)

     
     },
    0x4749 => {
     debug_print!("RatingPercent ");

         print_data(&data)

     
     },
    0x7000 => {
     debug_print!("SonyRawFileType ");

         print_data(&data)

     
     },
    0x7010 => {
     debug_print!("SonyToneCurve ");

         print_data(&data)

     
     },
    0x7031 => {
     debug_print!("VignettingCorrection ");

         print_data(&data)

     
     },
    0x7032 => {
     debug_print!("VignettingCorrParams ");

         print_data(&data)

     
     },
    0x7034 => {
     debug_print!("ChromaticAberrationCorrection ");

         print_data(&data)

     
     },
    0x7035 => {
     debug_print!("ChromaticAberrationCorrParams ");

         print_data(&data)

     
     },
    0x7036 => {
     debug_print!("DistortionCorrection ");

         print_data(&data)

     
     },
    0x7037 => {
     debug_print!("DistortionCorrParams ");

         print_data(&data)

     
     },
    0x74c7 => {
     debug_print!("SonyCropTopLeft ");

         print_data(&data)

     
     },
    0x74c8 => {
     debug_print!("SonyCropSize ");

         print_data(&data)

     
     },
    0x800d => {
     debug_print!("ImageID ");

         print_data(&data)

     
     },
    0x80a3 => {
     debug_print!("WangTag1 ");

         print_data(&data)

     
     },
    0x80a4 => {
     debug_print!("WangAnnotation ");

         print_data(&data)

     
     },
    0x80a5 => {
     debug_print!("WangTag3 ");

         print_data(&data)

     
     },
    0x80a6 => {
     debug_print!("WangTag4 ");

         print_data(&data)

     
     },
    0x80b9 => {
     debug_print!("ImageReferencePoints ");

         print_data(&data)

     
     },
    0x80ba => {
     debug_print!("RegionXformTackPoint ");

         print_data(&data)

     
     },
    0x80bb => {
     debug_print!("WarpQuadrilateral ");

         print_data(&data)

     
     },
    0x80bc => {
     debug_print!("AffineTransformMat ");

         print_data(&data)

     
     },
    0x80e3 => {
     debug_print!("Matteing ");

         print_data(&data)

     
     },
    0x80e4 => {
     debug_print!("DataType ");

         print_data(&data)

     
     },
    0x80e5 => {
     debug_print!("ImageDepth ");

         print_data(&data)

     
     },
    0x80e6 => {
     debug_print!("TileDepth ");

         print_data(&data)

     
     },
    0x8214 => {
     debug_print!("ImageFullWidth ");

         print_data(&data)

     
     },
    0x8215 => {
     debug_print!("ImageFullHeight ");

         print_data(&data)

     
     },
    0x8216 => {
     debug_print!("TextureFormat ");

         print_data(&data)

     
     },
    0x8217 => {
     debug_print!("WrapModes ");

         print_data(&data)

     
     },
    0x8218 => {
     debug_print!("FovCot ");

         print_data(&data)

     
     },
    0x8219 => {
     debug_print!("MatrixWorldToScreen ");

         print_data(&data)

     
     },
    0x821a => {
     debug_print!("MatrixWorldToCamera ");

         print_data(&data)

     
     },
    0x827d => {
     debug_print!("Model2 ");

         print_data(&data)

     
     },
    0x828d => {
     debug_print!("CFARepeatPatternDim ");

         print_data(&data)

     
     },
    0x828e => {
     debug_print!("CFAPattern2 ");

         print_data(&data)

     
     },
    0x828f => {
     debug_print!("BatteryLevel ");

         print_data(&data)

     
     },
    0x8290 => {
     debug_print!("KodakIFD ");

         print_data(&data)

     
     },
    0x8298 => {
     debug_print!("Copyright ");

         print_data(&data)

     
     },
    0x829a => {
     debug_print!("ExposureTime ");

         print_data(&data)

     
     },
    0x829d => {
     debug_print!("FNumber ");

         print_data(&data)

     
     },
    0x82a5 => {
     debug_print!("MDFileTag ");

         print_data(&data)

     
     },
    0x82a6 => {
     debug_print!("MDScalePixel ");

         print_data(&data)

     
     },
    0x82a7 => {
     debug_print!("MDColorTable ");

         print_data(&data)

     
     },
    0x82a8 => {
     debug_print!("MDLabName ");

         print_data(&data)

     
     },
    0x82a9 => {
     debug_print!("MDSampleInfo ");

         print_data(&data)

     
     },
    0x82aa => {
     debug_print!("MDPrepDate ");

         print_data(&data)

     
     },
    0x82ab => {
     debug_print!("MDPrepTime ");

         print_data(&data)

     
     },
    0x82ac => {
     debug_print!("MDFileUnits ");

         print_data(&data)

     
     },
    0x830e => {
     debug_print!("PixelScale ");

         print_data(&data)

     
     },
    0x8335 => {
     debug_print!("AdventScale ");

         print_data(&data)

     
     },
    0x8336 => {
     debug_print!("AdventRevision ");

         print_data(&data)

     
     },
    0x835c => {
     debug_print!("UIC1Tag ");

         print_data(&data)

     
     },
    0x835d => {
     debug_print!("UIC2Tag ");

         print_data(&data)

     
     },
    0x835e => {
     debug_print!("UIC3Tag ");

         print_data(&data)

     
     },
    0x835f => {
     debug_print!("UIC4Tag ");

         print_data(&data)

     
     },
    0x83bb => {
     debug_print!("IPTC-NAA ");

         print_data(&data)

     
     },
    0x847e => {
     debug_print!("IntergraphPacketData ");

         print_data(&data)

     
     },
    0x847f => {
     debug_print!("IntergraphFlagRegisters ");

         print_data(&data)

     
     },
    0x8480 => {
     debug_print!("IntergraphMatrix ");

         print_data(&data)

     
     },
    0x8481 => {
     debug_print!("INGRReserved ");

         print_data(&data)

     
     },
    0x8482 => {
     debug_print!("ModelTiePoint ");

         print_data(&data)

     
     },
    0x84e0 => {
     debug_print!("Site ");

         print_data(&data)

     
     },
    0x84e1 => {
     debug_print!("ColorSequence ");

         print_data(&data)

     
     },
    0x84e2 => {
     debug_print!("IT8Header ");

         print_data(&data)

     
     },
    0x84e3 => {
     debug_print!("RasterPadding ");

         print_data(&data)

     
     },
    0x84e4 => {
     debug_print!("BitsPerRunLength ");

         print_data(&data)

     
     },
    0x84e5 => {
     debug_print!("BitsPerExtendedRunLength ");

         print_data(&data)

     
     },
    0x84e6 => {
     debug_print!("ColorTable ");

         print_data(&data)

     
     },
    0x84e7 => {
     debug_print!("ImageColorIndicator ");

         print_data(&data)

     
     },
    0x84e8 => {
     debug_print!("BackgroundColorIndicator ");

         print_data(&data)

     
     },
    0x84e9 => {
     debug_print!("ImageColorValue ");

         print_data(&data)

     
     },
    0x84ea => {
     debug_print!("BackgroundColorValue ");

         print_data(&data)

     
     },
    0x84eb => {
     debug_print!("PixelIntensityRange ");

         print_data(&data)

     
     },
    0x84ec => {
     debug_print!("TransparencyIndicator ");

         print_data(&data)

     
     },
    0x84ed => {
     debug_print!("ColorCharacterization ");

         print_data(&data)

     
     },
    0x84ee => {
     debug_print!("HCUsage ");

         print_data(&data)

     
     },
    0x84ef => {
     debug_print!("TrapIndicator ");

         print_data(&data)

     
     },
    0x84f0 => {
     debug_print!("CMYKEquivalent ");

         print_data(&data)

     
     },
    0x8546 => {
     debug_print!("SEMInfo ");

         print_data(&data)

     
     },
    0x8568 => {
     debug_print!("AFCP_IPTC ");

         print_data(&data)

     
     },
    0x85b8 => {
     debug_print!("PixelMagicJBIGOptions ");

         print_data(&data)

     
     },
    0x85d7 => {
     debug_print!("JPLCartoIFD ");

         print_data(&data)

     
     },
    0x85d8 => {
     debug_print!("ModelTransform ");

         print_data(&data)

     
     },
    0x8602 => {
     debug_print!("WB_GRGBLevels ");

         print_data(&data)

     
     },
    0x8606 => {
     debug_print!("LeafData ");

         print_data(&data)

     
     },
    0x8649 => {
     debug_print!("PhotoshopSettings ");

         print_data(&data)

     
     },
    0x8769 => {
     debug_print!("ExifOffset ");

         print_data(&data)

     
     },
    0x8773 => {
     debug_print!("ICC_Profile ");

         print_data(&data)

     
     },
    0x877f => {
     debug_print!("TIFF_FXExtensions ");

         print_data(&data)

     
     },
    0x8780 => {
     debug_print!("MultiProfiles ");

         print_data(&data)

     
     },
    0x8781 => {
     debug_print!("SharedData ");

         print_data(&data)

     
     },
    0x8782 => {
     debug_print!("T88Options ");

         print_data(&data)

     
     },
    0x87ac => {
     debug_print!("ImageLayer ");

         print_data(&data)

     
     },
    0x87af => {
     debug_print!("GeoTiffDirectory ");

         print_data(&data)

     
     },
    0x87b0 => {
     debug_print!("GeoTiffDoubleParams ");

         print_data(&data)

     
     },
    0x87b1 => {
     debug_print!("GeoTiffAsciiParams ");

         print_data(&data)

     
     },
    0x87be => {
     debug_print!("JBIGOptions ");

         print_data(&data)

     
     },
    0x8822 => {
     debug_print!("ExposureProgram ");

         print_data(&data)

     
     },
    0x8824 => {
     debug_print!("SpectralSensitivity ");

         print_data(&data)

     
     },
    0x8825 => {
     debug_print!("GPSInfo ");

         print_data(&data)

     
     },
    0x8827 => {
     debug_print!("ISO ");

         print_data(&data)

     
     },
    0x8828 => {
     debug_print!("Opto-ElectricConvFactor ");

         print_data(&data)

     
     },
    0x8829 => {
     debug_print!("Interlace ");

         print_data(&data)

     
     },
    0x882a => {
     debug_print!("TimeZoneOffset ");

         print_data(&data)

     
     },
    0x882b => {
     debug_print!("SelfTimerMode ");

         print_data(&data)

     
     },
    0x8830 => {
     debug_print!("SensitivityType ");

         print_data(&data)

     
     },
    0x8831 => {
     debug_print!("StandardOutputSensitivity ");

         print_data(&data)

     
     },
    0x8832 => {
     debug_print!("RecommendedExposureIndex ");

         print_data(&data)

     
     },
    0x8833 => {
     debug_print!("ISOSpeed ");

         print_data(&data)

     
     },
    0x8834 => {
     debug_print!("ISOSpeedLatitudeyyy ");

         print_data(&data)

     
     },
    0x8835 => {
     debug_print!("ISOSpeedLatitudezzz ");

         print_data(&data)

     
     },
    0x885c => {
     debug_print!("FaxRecvParams ");

         print_data(&data)

     
     },
    0x885d => {
     debug_print!("FaxSubAddress ");

         print_data(&data)

     
     },
    0x885e => {
     debug_print!("FaxRecvTime ");

         print_data(&data)

     
     },
    0x8871 => {
     debug_print!("FedexEDR ");

         print_data(&data)

     
     },
    0x888a => {
     debug_print!("LeafSubIFD ");

         print_data(&data)

     
     },
    0x9000 => {
     debug_print!("ExifVersion ");

         print_data(&data)

     
     },
    0x9003 => {
     debug_print!("DateTimeOriginal ");

         print_data(&data)

     
     },
    0x9004 => {
     debug_print!("CreateDate ");

         print_data(&data)

     
     },
    0x9009 => {
     debug_print!("GooglePlusUploadCode ");

         print_data(&data)

     
     },
    0x9010 => {
     debug_print!("OffsetTime ");

         print_data(&data)

     
     },
    0x9011 => {
     debug_print!("OffsetTimeOriginal ");

         print_data(&data)

     
     },
    0x9012 => {
     debug_print!("OffsetTimeDigitized ");

         print_data(&data)

     
     },
    0x9101 => {
     debug_print!("ComponentsConfiguration ");

         print_data(&data)

     
     },
    0x9102 => {
     debug_print!("CompressedBitsPerPixel ");

         print_data(&data)

     
     },
    0x9201 => {
     debug_print!("ShutterSpeedValue ");

         print_data(&data)

     
     },
    0x9202 => {
     debug_print!("ApertureValue ");

         print_data(&data)

     
     },
    0x9203 => {
     debug_print!("BrightnessValue ");

         print_data(&data)

     
     },
    0x9204 => {
     debug_print!("ExposureCompensation ");

         print_data(&data)

     
     },
    0x9205 => {
     debug_print!("MaxApertureValue ");

         print_data(&data)

     
     },
    0x9206 => {
     debug_print!("SubjectDistance ");

         print_data(&data)

     
     },
    0x9207 => {
     debug_print!("MeteringMode ");

         print_data(&data)

     
     },
    0x9208 => {
     debug_print!("LightSource ");

         print_data(&data)

     
     },
    0x9209 => {
     debug_print!("Flash ");

         print_data(&data)

     
     },
    0x920a => {
     debug_print!("FocalLength ");

         print_data(&data)

     
     },
    0x920b => {
     debug_print!("FlashEnergy ");

         print_data(&data)

     
     },
    0x920c => {
     debug_print!("SpatialFrequencyResponse ");

         print_data(&data)

     
     },
    0x920d => {
     debug_print!("Noise ");

         print_data(&data)

     
     },
    0x920e => {
     debug_print!("FocalPlaneXResolution ");

         print_data(&data)

     
     },
    0x920f => {
     debug_print!("FocalPlaneYResolution ");

         print_data(&data)

     
     },
    0x9210 => {
     debug_print!("FocalPlaneResolutionUnit ");

         print_data(&data)

     
     },
    0x9211 => {
     debug_print!("ImageNumber ");

         print_data(&data)

     
     },
    0x9212 => {
     debug_print!("SecurityClassification ");

         print_data(&data)

     
     },
    0x9213 => {
     debug_print!("ImageHistory ");

         print_data(&data)

     
     },
    0x9214 => {
     debug_print!("SubjectArea ");

         print_data(&data)

     
     },
    0x9215 => {
     debug_print!("ExposureIndex ");

         print_data(&data)

     
     },
    0x9216 => {
     debug_print!("TIFF-EPStandardID ");

         print_data(&data)

     
     },
    0x9217 => {
     debug_print!("SensingMethod ");

         print_data(&data)

     
     },
    0x923a => {
     debug_print!("CIP3DataFile ");

         print_data(&data)

     
     },
    0x923b => {
     debug_print!("CIP3Sheet ");

         print_data(&data)

     
     },
    0x923c => {
     debug_print!("CIP3Side ");

         print_data(&data)

     
     },
    0x923f => {
     debug_print!("StoNits ");

         print_data(&data)

     
     },
    0x927c => {
        debug_print!("MakerNoteApple ");
        match data {
            DataPack::Undef(d) => {

            let string = read_string(d, 0, d.len());
            debug_print!("{}",string)
            },
            _ => {}
        }

         print_data(&data)

     
     },
    0x9286 => {
     debug_print!("UserComment ");

         print_data(&data)

     
     },
    0x9290 => {
     debug_print!("SubSecTime ");

         print_data(&data)

     
     },
    0x9291 => {
     debug_print!("SubSecTimeOriginal ");

         print_data(&data)

     
     },
    0x9292 => {
     debug_print!("SubSecTimeDigitized ");

         print_data(&data)

     
     },
    0x932f => {
     debug_print!("MSDocumentText ");

         print_data(&data)

     
     },
    0x9330 => {
     debug_print!("MSPropertySetStorage ");

         print_data(&data)

     
     },
    0x9331 => {
     debug_print!("MSDocumentTextPosition ");

         print_data(&data)

     
     },
    0x935c => {
     debug_print!("ImageSourceData ");

         print_data(&data)

     
     },
    0x9400 => {
     debug_print!("AmbientTemperature ");

         print_data(&data)

     
     },
    0x9401 => {
     debug_print!("Humidity ");

         print_data(&data)

     
     },
    0x9402 => {
     debug_print!("Pressure ");

         print_data(&data)

     
     },
    0x9403 => {
     debug_print!("WaterDepth ");

         print_data(&data)

     
     },
    0x9404 => {
     debug_print!("Acceleration ");

         print_data(&data)

     
     },
    0x9405 => {
     debug_print!("CameraElevationAngle ");

         print_data(&data)

     
     },
    0x9c9b => {
     debug_print!("XPTitle ");

         print_data(&data)

     
     },
    0x9c9c => {
     debug_print!("XPComment ");

         print_data(&data)

     
     },
    0x9c9d => {
     debug_print!("XPAuthor ");

         print_data(&data)

     
     },
    0x9c9e => {
     debug_print!("XPKeywords ");

         print_data(&data)

     
     },
    0x9c9f => {
     debug_print!("XPSubject ");

         print_data(&data)

     
     },
    0xa000 => {
     debug_print!("FlashpixVersion ");

         print_data(&data)

     
     },
    0xa001 => {
     debug_print!("ColorSpace ");

         print_data(&data)

     
     },
    0xa002 => {
     debug_print!("ExifImageWidth ");

         print_data(&data)

     
     },
    0xa003 => {
     debug_print!("ExifImageHeight ");

         print_data(&data)

     
     },
    0xa004 => {
     debug_print!("RelatedSoundFile ");

         print_data(&data)

     
     },
    0xa005 => {
     debug_print!("InteropOffset ");

         print_data(&data)

     
     },
    0xa010 => {
     debug_print!("SamsungRawPointersOffset ");

         print_data(&data)

     
     },
    0xa011 => {
     debug_print!("SamsungRawPointersLength ");

         print_data(&data)

     
     },
    0xa101 => {
     debug_print!("SamsungRawByteOrder ");

         print_data(&data)

     
     },
    0xa102 => {
     debug_print!("SamsungRawUnknown? ");

         print_data(&data)

     
     },
    0xa20b => {
     debug_print!("FlashEnergy ");

         print_data(&data)

     
     },
    0xa20c => {
     debug_print!("SpatialFrequencyResponse ");

         print_data(&data)

     
     },
    0xa20d => {
     debug_print!("Noise ");

         print_data(&data)

     
     },
    0xa20e => {
     debug_print!("FocalPlaneXResolution ");

         print_data(&data)

     
     },
    0xa20f => {
     debug_print!("FocalPlaneYResolution ");

         print_data(&data)

     
     },
    0xa210 => {
     debug_print!("FocalPlaneResolutionUnit ");

         print_data(&data)

     
     },
    0xa211 => {
     debug_print!("ImageNumber ");

         print_data(&data)

     
     },
    0xa212 => {
     debug_print!("SecurityClassification ");

         print_data(&data)

     
     },
    0xa213 => {
     debug_print!("ImageHistory ");

         print_data(&data)

     
     },
    0xa214 => {
     debug_print!("SubjectLocation ");

         print_data(&data)

     
     },
    0xa215 => {
     debug_print!("ExposureIndex ");

         print_data(&data)

     
     },
    0xa216 => {
     debug_print!("TIFF-EPStandardID ");

         print_data(&data)

     
     },
    0xa217 => {
     debug_print!("SensingMethod ");

         print_data(&data)

     
     },
    0xa300 => {
     debug_print!("FileSource ");

         print_data(&data)

     
     },
    0xa301 => {
     debug_print!("SceneType ");

         print_data(&data)

     
     },
    0xa302 => {
     debug_print!("CFAPattern ");

         print_data(&data)

     
     },
    0xa401 => {
     debug_print!("CustomRendered ");

         print_data(&data)

     
     },
    0xa402 => {
     debug_print!("ExposureMode ");

         print_data(&data)

     
     },
    0xa403 => {
     debug_print!("WhiteBalance ");

         print_data(&data)

     
     },
    0xa404 => {
     debug_print!("DigitalZoomRatio ");

         print_data(&data)

     
     },
    0xa405 => {
     debug_print!("FocalLengthIn35mmFormat ");

         print_data(&data)

     
     },
    0xa406 => {
     debug_print!("SceneCaptureType ");

         print_data(&data)

     
     },
    0xa407 => {
     debug_print!("GainControl ");

         print_data(&data)

     
     },
    0xa408 => {
     debug_print!("Contrast ");

         print_data(&data)

     
     },
    0xa409 => {
     debug_print!("Saturation ");

         print_data(&data)

     
     },
    0xa40a => {
     debug_print!("Sharpness ");

         print_data(&data)

     
     },
    0xa40b => {
     debug_print!("DeviceSettingDescription ");

         print_data(&data)

     
     },
    0xa40c => {
     debug_print!("SubjectDistanceRange ");

         print_data(&data)

     
     },
    0xa420 => {
     debug_print!("ImageUniqueID ");

         print_data(&data)

     
     },
    0xa430 => {
     debug_print!("OwnerName ");

         print_data(&data)

     
     },
    0xa431 => {
     debug_print!("SerialNumber ");

         print_data(&data)

     
     },
    0xa432 => {
     debug_print!("LensInfo ");

         print_data(&data)

     
     },
    0xa433 => {
     debug_print!("LensMake ");

         print_data(&data)

     
     },
    0xa434 => {
     debug_print!("LensModel ");

         print_data(&data)

     
     },
    0xa435 => {
     debug_print!("LensSerialNumber ");

         print_data(&data)

     
     },
    0xa460 => {
     debug_print!("CompositeImage ");

         print_data(&data)

     
     },
    0xa461 => {
     debug_print!("CompositeImageCount ");

         print_data(&data)

     
     },
    0xa462 => {
     debug_print!("CompositeImageExposureTimes ");

         print_data(&data)

     
     },
    0xa480 => {
     debug_print!("GDALMetadata ");

         print_data(&data)

     
     },
    0xa481 => {
     debug_print!("GDALNoData ");

         print_data(&data)

     
     },
    0xa500 => {
     debug_print!("Gamma ");

         print_data(&data)

     
     },
    0xafc0 => {
     debug_print!("ExpandSoftware ");

         print_data(&data)

     
     },
    0xafc1 => {
     debug_print!("ExpandLens ");

         print_data(&data)

     
     },
    0xafc2 => {
     debug_print!("ExpandFilm ");

         print_data(&data)

     
     },
    0xafc3 => {
     debug_print!("ExpandFilterLens ");

         print_data(&data)

     
     },
    0xafc4 => {
     debug_print!("ExpandScanner ");

         print_data(&data)

     
     },
    0xafc5 => {
     debug_print!("ExpandFlashLamp ");

         print_data(&data)

     
     },
    0xb4c3 => {
     debug_print!("HasselbladRawImage ");

         print_data(&data)

     
     },
    0xbc01 => {
     debug_print!("PixelFormat ");

         print_data(&data)

     
     },
    0xbc02 => {
     debug_print!("Transformation ");

         print_data(&data)

     
     },
    0xbc03 => {
     debug_print!("Uncompressed ");

         print_data(&data)

     
     },
    0xbc04 => {
     debug_print!("ImageType ");

         print_data(&data)

     
     },
    0xbc80 => {
     debug_print!("ImageWidth ");

         print_data(&data)

     
     },
    0xbc81 => {
     debug_print!("ImageHeight ");

         print_data(&data)

     
     },
    0xbc82 => {
     debug_print!("WidthResolution ");

         print_data(&data)

     
     },
    0xbc83 => {
     debug_print!("HeightResolution ");

         print_data(&data)

     
     },
    0xbcc0 => {
     debug_print!("ImageOffset ");

         print_data(&data)

     
     },
    0xbcc1 => {
     debug_print!("ImageByteCount ");

         print_data(&data)

     
     },
    0xbcc2 => {
     debug_print!("AlphaOffset ");

         print_data(&data)

     
     },
    0xbcc3 => {
     debug_print!("AlphaByteCount ");

         print_data(&data)

     
     },
    0xbcc4 => {
     debug_print!("ImageDataDiscard ");

         print_data(&data)

     
     },
    0xbcc5 => {
     debug_print!("AlphaDataDiscard ");

         print_data(&data)

     
     },
    0xc427 => {
     debug_print!("OceScanjobDesc ");

         print_data(&data)

     
     },
    0xc428 => {
     debug_print!("OceApplicationSelector ");

         print_data(&data)

     
     },
    0xc429 => {
     debug_print!("OceIDNumber ");

         print_data(&data)

     
     },
    0xc42a => {
     debug_print!("OceImageLogic ");

         print_data(&data)

     
     },
    0xc44f => {
     debug_print!("Annotations ");

         print_data(&data)

     
     },
    0xc4a5 => {
     debug_print!("PrintIM ");

         print_data(&data)

     
     },
    0xc51b => {
     debug_print!("HasselbladExif ");

         print_data(&data)

     
     },
    0xc573 => {
     debug_print!("OriginalFileName ");

         print_data(&data)

     
     },
    0xc580 => {
     debug_print!("USPTOOriginalContentType ");

         print_data(&data)

     
     },
    0xc5e0 => {
     debug_print!("CR2CFAPattern ");

         print_data(&data)

     
     },
    0xc612 => {
     debug_print!("DNGVersion ");

         print_data(&data)

     
     },
    0xc613 => {
     debug_print!("DNGBackwardVersion ");

         print_data(&data)

     
     },
    0xc614 => {
     debug_print!("UniqueCameraModel ");

         print_data(&data)

     
     },
    0xc615 => {
     debug_print!("LocalizedCameraModel ");

         print_data(&data)

     
     },
    0xc616 => {
     debug_print!("CFAPlaneColor ");

         print_data(&data)

     
     },
    0xc617 => {
     debug_print!("CFALayout ");

         print_data(&data)

     
     },
    0xc618 => {
     debug_print!("LinearizationTable ");

         print_data(&data)

     
     },
    0xc619 => {
     debug_print!("BlackLevelRepeatDim ");

         print_data(&data)

     
     },
    0xc61a => {
     debug_print!("BlackLevel ");

         print_data(&data)

     
     },
    0xc61b => {
     debug_print!("BlackLevelDeltaH ");

         print_data(&data)

     
     },
    0xc61c => {
     debug_print!("BlackLevelDeltaV ");

         print_data(&data)

     
     },
    0xc61d => {
     debug_print!("WhiteLevel ");

         print_data(&data)

     
     },
    0xc61e => {
     debug_print!("DefaultScale ");

         print_data(&data)

     
     },
    0xc61f => {
     debug_print!("DefaultCropOrigin ");

         print_data(&data)

     
     },
    0xc620 => {
     debug_print!("DefaultCropSize ");

         print_data(&data)

     
     },
    0xc621 => {
     debug_print!("ColorMatrix1 ");

         print_data(&data)

     
     },
    0xc622 => {
     debug_print!("ColorMatrix2 ");

         print_data(&data)

     
     },
    0xc623 => {
     debug_print!("CameraCalibration1 ");

         print_data(&data)

     
     },
    0xc624 => {
     debug_print!("CameraCalibration2 ");

         print_data(&data)

     
     },
    0xc625 => {
     debug_print!("ReductionMatrix1 ");

         print_data(&data)

     
     },
    0xc626 => {
     debug_print!("ReductionMatrix2 ");

         print_data(&data)

     
     },
    0xc627 => {
     debug_print!("AnalogBalance ");

         print_data(&data)

     
     },
    0xc628 => {
     debug_print!("AsShotNeutral ");

         print_data(&data)

     
     },
    0xc629 => {
     debug_print!("AsShotWhiteXY ");

         print_data(&data)

     
     },
    0xc62a => {
     debug_print!("BaselineExposure ");

         print_data(&data)

     
     },
    0xc62b => {
     debug_print!("BaselineNoise ");

         print_data(&data)

     
     },
    0xc62c => {
     debug_print!("BaselineSharpness ");

         print_data(&data)

     
     },
    0xc62d => {
     debug_print!("BayerGreenSplit ");

         print_data(&data)

     
     },
    0xc62e => {
     debug_print!("LinearResponseLimit ");

         print_data(&data)

     
     },
    0xc62f => {
     debug_print!("CameraSerialNumber ");

         print_data(&data)

     
     },
    0xc630 => {
     debug_print!("DNGLensInfo ");

         print_data(&data)

     
     },
    0xc631 => {
     debug_print!("ChromaBlurRadius ");

         print_data(&data)

     
     },
    0xc632 => {
     debug_print!("AntiAliasStrength ");

         print_data(&data)

     
     },
    0xc633 => {
     debug_print!("ShadowScale ");

         print_data(&data)

     
     },
    0xc634 => {
     debug_print!("SR2Private ");

         print_data(&data)

     
     },
    0xc635 => {
     debug_print!("MakerNoteSafety ");

         print_data(&data)

     
     },
    0xc640 => {
     debug_print!("RawImageSegmentation ");

         print_data(&data)

     
     },
    0xc65a => {
     debug_print!("CalibrationIlluminant1 ");

         print_data(&data)

     
     },
    0xc65b => {
     debug_print!("CalibrationIlluminant2 ");

         print_data(&data)

     
     },
    0xc65c => {
     debug_print!("BestQualityScale ");

         print_data(&data)

     
     },
    0xc65d => {
     debug_print!("RawDataUniqueID ");

         print_data(&data)

     
     },
    0xc660 => {
     debug_print!("AliasLayerMetadata ");

         print_data(&data)

     
     },
    0xc68b => {
     debug_print!("OriginalRawFileName ");

         print_data(&data)

     
     },
    0xc68c => {
     debug_print!("OriginalRawFileData ");

         print_data(&data)

     
     },
    0xc68d => {
     debug_print!("ActiveArea ");

         print_data(&data)

     
     },
    0xc68e => {
     debug_print!("MaskedAreas ");

         print_data(&data)

     
     },
    0xc68f => {
     debug_print!("AsShotICCProfile ");

         print_data(&data)

     
     },
    0xc690 => {
     debug_print!("AsShotPreProfileMatrix ");

         print_data(&data)

     
     },
    0xc691 => {
     debug_print!("CurrentICCProfile ");

         print_data(&data)

     
     },
    0xc692 => {
     debug_print!("CurrentPreProfileMatrix ");

         print_data(&data)

     
     },
    0xc6bf => {
     debug_print!("ColorimetricReference ");

         print_data(&data)

     
     },
    0xc6c5 => {
     debug_print!("SRawType ");

         print_data(&data)

     
     },
    0xc6d2 => {
     debug_print!("PanasonicTitle ");

         print_data(&data)

     
     },
    0xc6d3 => {
     debug_print!("PanasonicTitle2 ");

         print_data(&data)

     
     },
    0xc6f3 => {
     debug_print!("CameraCalibrationSig ");

         print_data(&data)

     
     },
    0xc6f4 => {
     debug_print!("ProfileCalibrationSig ");

         print_data(&data)

     
     },
    0xc6f5 => {
     debug_print!("ProfileIFD ");

         print_data(&data)

     
     },
    0xc6f6 => {
     debug_print!("AsShotProfileName ");

         print_data(&data)

     
     },
    0xc6f7 => {
     debug_print!("NoiseReductionApplied ");

         print_data(&data)

     
     },
    0xc6f8 => {
     debug_print!("ProfileName ");

         print_data(&data)

     
     },
    0xc6f9 => {
     debug_print!("ProfileHueSatMapDims ");

         print_data(&data)

     
     },
    0xc6fa => {
     debug_print!("ProfileHueSatMapData1 ");

         print_data(&data)

     
     },
    0xc6fb => {
     debug_print!("ProfileHueSatMapData2 ");

         print_data(&data)

     
     },
    0xc6fc => {
     debug_print!("ProfileToneCurve ");

         print_data(&data)

     
     },
    0xc6fd => {
     debug_print!("ProfileEmbedPolicy ");

         print_data(&data)

     
     },
    0xc6fe => {
     debug_print!("ProfileCopyright ");

         print_data(&data)

     
     },
    0xc714 => {
     debug_print!("ForwardMatrix1 ");

         print_data(&data)

     
     },
    0xc715 => {
     debug_print!("ForwardMatrix2 ");

         print_data(&data)

     
     },
    0xc716 => {
     debug_print!("PreviewApplicationName ");

         print_data(&data)

     
     },
    0xc717 => {
     debug_print!("PreviewApplicationVersion ");

         print_data(&data)

     
     },
    0xc718 => {
     debug_print!("PreviewSettingsName ");

         print_data(&data)

     
     },
    0xc719 => {
     debug_print!("PreviewSettingsDigest ");

         print_data(&data)

     
     },
    0xc71a => {
     debug_print!("PreviewColorSpace ");

         print_data(&data)

     
     },
    0xc71b => {
     debug_print!("PreviewDateTime ");

         print_data(&data)

     
     },
    0xc71c => {
     debug_print!("RawImageDigest ");

         print_data(&data)

     
     },
    0xc71d => {
     debug_print!("OriginalRawFileDigest ");

         print_data(&data)

     
     },
    0xc71e => {
     debug_print!("SubTileBlockSize ");

         print_data(&data)

     
     },
    0xc71f => {
     debug_print!("RowInterleaveFactor ");

         print_data(&data)

     
     },
    0xc725 => {
     debug_print!("ProfileLookTableDims ");

         print_data(&data)

     
     },
    0xc726 => {
     debug_print!("ProfileLookTableData ");

         print_data(&data)

     
     },
    0xc740 => {
     debug_print!("OpcodeList1 ");

         print_data(&data)

     
     },
    0xc741 => {
     debug_print!("OpcodeList2 ");

         print_data(&data)

     
     },
    0xc74e => {
     debug_print!("OpcodeList3 ");

         print_data(&data)

     
     },
    0xc761 => {
     debug_print!("NoiseProfile ");

         print_data(&data)

     
     },
    0xc763 => {
     debug_print!("TimeCodes ");

         print_data(&data)

     
     },
    0xc764 => {
     debug_print!("FrameRate ");

         print_data(&data)

     
     },
    0xc772 => {
     debug_print!("TStop ");

         print_data(&data)

     
     },
    0xc789 => {
     debug_print!("ReelName ");

         print_data(&data)

     
     },
    0xc791 => {
     debug_print!("OriginalDefaultFinalSize ");

         print_data(&data)

     
     },
    0xc792 => {
     debug_print!("OriginalBestQualitySize ");

         print_data(&data)

     
     },
    0xc793 => {
     debug_print!("OriginalDefaultCropSize ");

         print_data(&data)

     
     },
    0xc7a1 => {
     debug_print!("CameraLabel ");

         print_data(&data)

     
     },
    0xc7a3 => {
     debug_print!("ProfileHueSatMapEncoding ");

         print_data(&data)

     
     },
    0xc7a4 => {
     debug_print!("ProfileLookTableEncoding ");

         print_data(&data)

     
     },
    0xc7a5 => {
     debug_print!("BaselineExposureOffset ");

         print_data(&data)

     
     },
    0xc7a6 => {
     debug_print!("DefaultBlackRender ");

         print_data(&data)

     
     },
    0xc7a7 => {
     debug_print!("NewRawImageDigest ");

         print_data(&data)

     
     },
    0xc7a8 => {
     debug_print!("RawToPreviewGain ");

         print_data(&data)

     
     },
    0xc7aa => {
     debug_print!("CacheVersion ");

         print_data(&data)

     
     },
    0xc7b5 => {
     debug_print!("DefaultUserCrop ");

         print_data(&data)

     
     },
    0xc7d5 => {
     debug_print!("NikonNEFInfo ");

         print_data(&data)

     
     },
    0xc7e9 => {
     debug_print!("DepthFormat ");

         print_data(&data)

     
     },
    0xc7ea => {
     debug_print!("DepthNear ");

         print_data(&data)

     
     },
    0xc7eb => {
     debug_print!("DepthFar ");

         print_data(&data)

     
     },
    0xc7ec => {
     debug_print!("DepthUnits ");

         print_data(&data)

     
     },
    0xc7ed => {
     debug_print!("DepthMeasureType ");

         print_data(&data)

     
     },
    0xc7ee => {
     debug_print!("EnhanceParams ");

         print_data(&data)

     
     },
    0xcd2d => {
     debug_print!("ProfileGainTableMap ");

         print_data(&data)

     
     },
    0xcd2e => {
     debug_print!("SemanticName ");

         print_data(&data)

     
     },
    0xcd30 => {
     debug_print!("SemanticInstanceIFD ");

         print_data(&data)

     
     },
    0xcd31 => {
     debug_print!("CalibrationIlluminant3 ");

         print_data(&data)

     
     },
    0xcd32 => {
     debug_print!("CameraCalibration3 ");

         print_data(&data)

     
     },
    0xcd33 => {
     debug_print!("ColorMatrix3 ");

         print_data(&data)

     
     },
    0xcd34 => {
     debug_print!("ForwardMatrix3 ");

         print_data(&data)

     
     },
    0xcd35 => {
     debug_print!("IlluminantData1 ");

         print_data(&data)

     
     },
    0xcd36 => {
     debug_print!("IlluminantData2 ");

         print_data(&data)

     
     },
    0xcd37 => {
     debug_print!("IlluminantData3 ");

         print_data(&data)

     
     },
    0xcd38 => {
     debug_print!("MaskSubArea ");

         print_data(&data)

     
     },
    0xcd39 => {
     debug_print!("ProfileHueSatMapData3 ");

         print_data(&data)

     
     },
    0xcd3a => {
     debug_print!("ReductionMatrix3 ");

         print_data(&data)

     
     },
    0xcd3b => {
     debug_print!("RGBTables ");

         print_data(&data)

     
     },
    0xea1c => {
     debug_print!("Padding ");

         print_data(&data)

     
     },
    0xea1d => {
     debug_print!("OffsetSchema ");

         print_data(&data)

     
     },
    0xfde8 => {
     debug_print!("OwnerName ");

         print_data(&data)

     
     },
    0xfde9 => {
     debug_print!("SerialNumber ");

         print_data(&data)

     
     },
    0xfdea => {
     debug_print!("Lens ");

         print_data(&data)

     
     },
    0xfe00 => {
     debug_print!("KDC_IFD ");

         print_data(&data)

     
     },
    0xfe4c => {
     debug_print!("RawFile ");

         print_data(&data)

     
     },
    0xfe4d => {
     debug_print!("Converter ");

         print_data(&data)

     
     },
    0xfe4e => {
     debug_print!("WhiteBalance ");

         print_data(&data)

     
     },
    0xfe51 => {
     debug_print!("Exposure ");

         print_data(&data)

     
     },
    0xfe52 => {
     debug_print!("Shadows ");

         print_data(&data)

     
     },
    0xfe53 => {
     debug_print!("Brightness ");

         print_data(&data)

     
     },
    0xfe54 => {
     debug_print!("Contrast ");

         print_data(&data)

     
     },
    0xfe55 => {
     debug_print!("Saturation ");

         print_data(&data)

     
     },
    0xfe56 => {
     debug_print!("Sharpness ");

         print_data(&data)

     
     },
    0xfe57 => {
     debug_print!("Smoothness ");

         print_data(&data)

     
     },
    0xfe58 => {
     debug_print!("MoireFilter ");

         print_data(&data)

     
     },
    _ => {
       print_data(&data)       
    },
    }
}
