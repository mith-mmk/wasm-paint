use crate::log;
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
        log("InteropIndex ");
        print_data(&data);
     },
    0x0002 => {
     log("InteropVersion ");

         print_data(&data)
     },
    0x000b => {
     log("ProcessingSoftware ");

         print_data(&data)

     
     },
    0x00fe => {
     log("SubfileType ");

         print_data(&data)

     
     },
    0x00ff => {
     log("OldSubfileType ");

         print_data(&data)

     
     },
    0x0100 => {
     log("ImageWidth ");

         print_data(&data)

     
     },
    0x0101 => {
     log("ImageHeight ");

         print_data(&data)

     
     },
    0x0102 => {
     log("BitsPerSample ");

         print_data(&data)

     
     },
    0x0103 => {
     log("Compression ");

         print_data(&data)

     
     },
    0x0106 => {
     log("PhotometricInterpretation ");

         print_data(&data)

     
     },
    0x0107 => {
     log("Thresholding ");

         print_data(&data)

     
     },
    0x0108 => {
     log("CellWidth ");

         print_data(&data)

     
     },
    0x0109 => {
     log("CellLength ");

         print_data(&data)

     
     },
    0x010a => {
     log("FillOrder ");

         print_data(&data)

     
     },
    0x010d => {
     log("DocumentName ");

         print_data(&data)

     
     },
    0x010e => {
     log("ImageDescription ");

         print_data(&data)

     
     },
    0x010f => {
     log("Make ");

         print_data(&data)

     
     },
    0x0110 => {
     log("Model ");

         print_data(&data)

     
     },
    0x0111 => {
     log("StripOffsets ");

         print_data(&data)

     
     },
    0x0112 => {
     log("Orientation ");

         print_data(&data)

     
     },
    0x0115 => {
     log("SamplesPerPixel ");

         print_data(&data)

     
     },
    0x0116 => {
     log("RowsPerStrip ");

         print_data(&data)

     
     },
    0x0117 => {
     log("StripByteCounts ");

         print_data(&data)

     
     },
    0x0118 => {
     log("MinSampleValue ");

         print_data(&data)

     
     },
    0x0119 => {
     log("MaxSampleValue ");

         print_data(&data)

     
     },
    0x011a => {
     log("XResolution ");

         print_data(&data)

     
     },
    0x011b => {
     log("YResolution ");

         print_data(&data)

     
     },
    0x011c => {
     log("PlanarConfiguration ");

         print_data(&data)

     
     },
    0x011d => {
     log("PageName ");

         print_data(&data)

     
     },
    0x011e => {
     log("XPosition ");

         print_data(&data)

     
     },
    0x011f => {
     log("YPosition ");

         print_data(&data)

     
     },
    0x0120 => {
     log("FreeOffsets ");

         print_data(&data)

     
     },
    0x0121 => {
     log("FreeByteCounts ");

         print_data(&data)

     
     },
    0x0122 => {
     log("GrayResponseUnit ");

         print_data(&data)

     
     },
    0x0123 => {
     log("GrayResponseCurve ");

         print_data(&data)

     
     },
    0x0124 => {
     log("T4Options ");

         print_data(&data)

     
     },
    0x0125 => {
     log("T6Options ");

         print_data(&data)

     
     },
    0x0128 => {
     log("ResolutionUnit ");

         print_data(&data)

     
     },
    0x0129 => {
     log("PageNumber ");

         print_data(&data)

     
     },
    0x012c => {
     log("ColorResponseUnit ");

         print_data(&data)

     
     },
    0x012d => {
     log("TransferFunction ");

         print_data(&data)

     
     },
    0x0131 => {
     log("Software ");

         print_data(&data)

     
     },
    0x0132 => {
     log("ModifyDate ");

         print_data(&data)

     
     },
    0x013b => {
     log("Artist ");

         print_data(&data)

     
     },
    0x013c => {
     log("HostComputer ");

         print_data(&data)

     
     },
    0x013d => {
     log("Predictor ");

         print_data(&data)

     
     },
    0x013e => {
     log("WhitePoint ");

         print_data(&data)

     
     },
    0x013f => {
     log("PrimaryChromaticities ");

         print_data(&data)

     
     },
    0x0140 => {
     log("ColorMap ");

         print_data(&data)

     
     },
    0x0141 => {
     log("HalftoneHints ");

         print_data(&data)

     
     },
    0x0142 => {
     log("TileWidth ");

         print_data(&data)

     
     },
    0x0143 => {
     log("TileLength ");

         print_data(&data)

     
     },
    0x0144 => {
     log("TileOffsets ");

         print_data(&data)

     
     },
    0x0145 => {
     log("TileByteCounts ");

         print_data(&data)

     
     },
    0x0146 => {
     log("BadFaxLines ");

         print_data(&data)

     
     },
    0x0147 => {
     log("CleanFaxData ");

         print_data(&data)

     
     },
    0x0148 => {
     log("ConsecutiveBadFaxLines ");

         print_data(&data)

     
     },
    0x014a => {
     log("SubIFD ");

         print_data(&data)

     
     },
    0x014c => {
     log("InkSet ");

         print_data(&data)

     
     },
    0x014d => {
     log("InkNames ");

         print_data(&data)

     
     },
    0x014e => {
     log("NumberofInks ");

         print_data(&data)

     
     },
    0x0150 => {
     log("DotRange ");

         print_data(&data)

     
     },
    0x0151 => {
     log("TargetPrinter ");

         print_data(&data)

     
     },
    0x0152 => {
     log("ExtraSamples ");

         print_data(&data)

     
     },
    0x0153 => {
     log("SampleFormat ");

         print_data(&data)

     
     },
    0x0154 => {
     log("SMinSampleValue ");

         print_data(&data)

     
     },
    0x0155 => {
     log("SMaxSampleValue ");

         print_data(&data)

     
     },
    0x0156 => {
     log("TransferRange ");

         print_data(&data)

     
     },
    0x0157 => {
     log("ClipPath ");

         print_data(&data)

     
     },
    0x0158 => {
     log("XClipPathUnits ");

         print_data(&data)

     
     },
    0x0159 => {
     log("YClipPathUnits ");

         print_data(&data)

     
     },
    0x015a => {
     log("Indexed ");

         print_data(&data)

     
     },
    0x015b => {
     log("JPEGTables ");

         print_data(&data)

     
     },
    0x015f => {
     log("OPIProxy ");

         print_data(&data)

     
     },
    0x0190 => {
     log("GlobalParametersIFD ");

         print_data(&data)

     
     },
    0x0191 => {
     log("ProfileType ");

         print_data(&data)

     
     },
    0x0192 => {
     log("FaxProfile ");

         print_data(&data)

     
     },
    0x0193 => {
     log("CodingMethods ");

         print_data(&data)

     
     },
    0x0194 => {
     log("VersionYear ");

         print_data(&data)

     
     },
    0x0195 => {
     log("ModeNumber ");

         print_data(&data)

     
     },
    0x01b1 => {
     log("Decode ");

         print_data(&data)

     
     },
    0x01b2 => {
     log("DefaultImageColor ");

         print_data(&data)

     
     },
    0x01b3 => {
     log("T82Options ");

         print_data(&data)

     
     },
    0x01b5 => {
     log("JPEGTables ");

         print_data(&data)

     
     },
    0x0200 => {
     log("JPEGProc ");

         print_data(&data)

     
     },
    0x0201 => {
     log("ThumbnailOffset ");

         print_data(&data)

     
     },
    0x0202 => {
     log("ThumbnailLength ");

         print_data(&data)

     
     },
    0x0203 => {
     log("JPEGRestartInterval ");

         print_data(&data)

     
     },
    0x0205 => {
     log("JPEGLosslessPredictors ");

         print_data(&data)

     
     },
    0x0206 => {
     log("JPEGPointTransforms ");

         print_data(&data)

     
     },
    0x0207 => {
     log("JPEGQTables ");

         print_data(&data)

     
     },
    0x0208 => {
     log("JPEGDCTables ");

         print_data(&data)

     
     },
    0x0209 => {
     log("JPEGACTables ");

         print_data(&data)

     
     },
    0x0211 => {
     log("YCbCrCoefficients ");

         print_data(&data)

     
     },
    0x0212 => {
     log("YCbCrSubSampling ");

         print_data(&data)

     
     },
    0x0213 => {
     log("YCbCrPositioning ");

         print_data(&data)

     
     },
    0x0214 => {
     log("ReferenceBlackWhite ");

         print_data(&data)

     
     },
    0x022f => {
     log("StripRowCounts ");

         print_data(&data)

     
     },
    0x02bc => {
     log("ApplicationNotes ");

         print_data(&data)

     
     },
    0x03e7 => {
     log("USPTOMiscellaneous ");

         print_data(&data)

     
     },
    0x1000 => {
     log("RelatedImageFileFormat ");

         print_data(&data)

     
     },
    0x1001 => {
     log("RelatedImageWidth ");

         print_data(&data)

     
     },
    0x1002 => {
     log("RelatedImageHeight ");

         print_data(&data)

     
     },
    0x4746 => {
     log("Rating ");

         print_data(&data)

     
     },
    0x4747 => {
     log("XP_DIP_XML ");

         print_data(&data)

     
     },
    0x4748 => {
     log("StitchInfo ");

         print_data(&data)

     
     },
    0x4749 => {
     log("RatingPercent ");

         print_data(&data)

     
     },
    0x7000 => {
     log("SonyRawFileType ");

         print_data(&data)

     
     },
    0x7010 => {
     log("SonyToneCurve ");

         print_data(&data)

     
     },
    0x7031 => {
     log("VignettingCorrection ");

         print_data(&data)

     
     },
    0x7032 => {
     log("VignettingCorrParams ");

         print_data(&data)

     
     },
    0x7034 => {
     log("ChromaticAberrationCorrection ");

         print_data(&data)

     
     },
    0x7035 => {
     log("ChromaticAberrationCorrParams ");

         print_data(&data)

     
     },
    0x7036 => {
     log("DistortionCorrection ");

         print_data(&data)

     
     },
    0x7037 => {
     log("DistortionCorrParams ");

         print_data(&data)

     
     },
    0x74c7 => {
     log("SonyCropTopLeft ");

         print_data(&data)

     
     },
    0x74c8 => {
     log("SonyCropSize ");

         print_data(&data)

     
     },
    0x800d => {
     log("ImageID ");

         print_data(&data)

     
     },
    0x80a3 => {
     log("WangTag1 ");

         print_data(&data)

     
     },
    0x80a4 => {
     log("WangAnnotation ");

         print_data(&data)

     
     },
    0x80a5 => {
     log("WangTag3 ");

         print_data(&data)

     
     },
    0x80a6 => {
     log("WangTag4 ");

         print_data(&data)

     
     },
    0x80b9 => {
     log("ImageReferencePoints ");

         print_data(&data)

     
     },
    0x80ba => {
     log("RegionXformTackPoint ");

         print_data(&data)

     
     },
    0x80bb => {
     log("WarpQuadrilateral ");

         print_data(&data)

     
     },
    0x80bc => {
     log("AffineTransformMat ");

         print_data(&data)

     
     },
    0x80e3 => {
     log("Matteing ");

         print_data(&data)

     
     },
    0x80e4 => {
     log("DataType ");

         print_data(&data)

     
     },
    0x80e5 => {
     log("ImageDepth ");

         print_data(&data)

     
     },
    0x80e6 => {
     log("TileDepth ");

         print_data(&data)

     
     },
    0x8214 => {
     log("ImageFullWidth ");

         print_data(&data)

     
     },
    0x8215 => {
     log("ImageFullHeight ");

         print_data(&data)

     
     },
    0x8216 => {
     log("TextureFormat ");

         print_data(&data)

     
     },
    0x8217 => {
     log("WrapModes ");

         print_data(&data)

     
     },
    0x8218 => {
     log("FovCot ");

         print_data(&data)

     
     },
    0x8219 => {
     log("MatrixWorldToScreen ");

         print_data(&data)

     
     },
    0x821a => {
     log("MatrixWorldToCamera ");

         print_data(&data)

     
     },
    0x827d => {
     log("Model2 ");

         print_data(&data)

     
     },
    0x828d => {
     log("CFARepeatPatternDim ");

         print_data(&data)

     
     },
    0x828e => {
     log("CFAPattern2 ");

         print_data(&data)

     
     },
    0x828f => {
     log("BatteryLevel ");

         print_data(&data)

     
     },
    0x8290 => {
     log("KodakIFD ");

         print_data(&data)

     
     },
    0x8298 => {
     log("Copyright ");

         print_data(&data)

     
     },
    0x829a => {
     log("ExposureTime ");

         print_data(&data)

     
     },
    0x829d => {
     log("FNumber ");

         print_data(&data)

     
     },
    0x82a5 => {
     log("MDFileTag ");

         print_data(&data)

     
     },
    0x82a6 => {
     log("MDScalePixel ");

         print_data(&data)

     
     },
    0x82a7 => {
     log("MDColorTable ");

         print_data(&data)

     
     },
    0x82a8 => {
     log("MDLabName ");

         print_data(&data)

     
     },
    0x82a9 => {
     log("MDSampleInfo ");

         print_data(&data)

     
     },
    0x82aa => {
     log("MDPrepDate ");

         print_data(&data)

     
     },
    0x82ab => {
     log("MDPrepTime ");

         print_data(&data)

     
     },
    0x82ac => {
     log("MDFileUnits ");

         print_data(&data)

     
     },
    0x830e => {
     log("PixelScale ");

         print_data(&data)

     
     },
    0x8335 => {
     log("AdventScale ");

         print_data(&data)

     
     },
    0x8336 => {
     log("AdventRevision ");

         print_data(&data)

     
     },
    0x835c => {
     log("UIC1Tag ");

         print_data(&data)

     
     },
    0x835d => {
     log("UIC2Tag ");

         print_data(&data)

     
     },
    0x835e => {
     log("UIC3Tag ");

         print_data(&data)

     
     },
    0x835f => {
     log("UIC4Tag ");

         print_data(&data)

     
     },
    0x83bb => {
     log("IPTC-NAA ");

         print_data(&data)

     
     },
    0x847e => {
     log("IntergraphPacketData ");

         print_data(&data)

     
     },
    0x847f => {
     log("IntergraphFlagRegisters ");

         print_data(&data)

     
     },
    0x8480 => {
     log("IntergraphMatrix ");

         print_data(&data)

     
     },
    0x8481 => {
     log("INGRReserved ");

         print_data(&data)

     
     },
    0x8482 => {
     log("ModelTiePoint ");

         print_data(&data)

     
     },
    0x84e0 => {
     log("Site ");

         print_data(&data)

     
     },
    0x84e1 => {
     log("ColorSequence ");

         print_data(&data)

     
     },
    0x84e2 => {
     log("IT8Header ");

         print_data(&data)

     
     },
    0x84e3 => {
     log("RasterPadding ");

         print_data(&data)

     
     },
    0x84e4 => {
     log("BitsPerRunLength ");

         print_data(&data)

     
     },
    0x84e5 => {
     log("BitsPerExtendedRunLength ");

         print_data(&data)

     
     },
    0x84e6 => {
     log("ColorTable ");

         print_data(&data)

     
     },
    0x84e7 => {
     log("ImageColorIndicator ");

         print_data(&data)

     
     },
    0x84e8 => {
     log("BackgroundColorIndicator ");

         print_data(&data)

     
     },
    0x84e9 => {
     log("ImageColorValue ");

         print_data(&data)

     
     },
    0x84ea => {
     log("BackgroundColorValue ");

         print_data(&data)

     
     },
    0x84eb => {
     log("PixelIntensityRange ");

         print_data(&data)

     
     },
    0x84ec => {
     log("TransparencyIndicator ");

         print_data(&data)

     
     },
    0x84ed => {
     log("ColorCharacterization ");

         print_data(&data)

     
     },
    0x84ee => {
     log("HCUsage ");

         print_data(&data)

     
     },
    0x84ef => {
     log("TrapIndicator ");

         print_data(&data)

     
     },
    0x84f0 => {
     log("CMYKEquivalent ");

         print_data(&data)

     
     },
    0x8546 => {
     log("SEMInfo ");

         print_data(&data)

     
     },
    0x8568 => {
     log("AFCP_IPTC ");

         print_data(&data)

     
     },
    0x85b8 => {
     log("PixelMagicJBIGOptions ");

         print_data(&data)

     
     },
    0x85d7 => {
     log("JPLCartoIFD ");

         print_data(&data)

     
     },
    0x85d8 => {
     log("ModelTransform ");

         print_data(&data)

     
     },
    0x8602 => {
     log("WB_GRGBLevels ");

         print_data(&data)

     
     },
    0x8606 => {
     log("LeafData ");

         print_data(&data)

     
     },
    0x8649 => {
     log("PhotoshopSettings ");

         print_data(&data)

     
     },
    0x8769 => {
     log("ExifOffset ");

         print_data(&data)

     
     },
    0x8773 => {
     log("ICC_Profile ");

         print_data(&data)

     
     },
    0x877f => {
     log("TIFF_FXExtensions ");

         print_data(&data)

     
     },
    0x8780 => {
     log("MultiProfiles ");

         print_data(&data)

     
     },
    0x8781 => {
     log("SharedData ");

         print_data(&data)

     
     },
    0x8782 => {
     log("T88Options ");

         print_data(&data)

     
     },
    0x87ac => {
     log("ImageLayer ");

         print_data(&data)

     
     },
    0x87af => {
     log("GeoTiffDirectory ");

         print_data(&data)

     
     },
    0x87b0 => {
     log("GeoTiffDoubleParams ");

         print_data(&data)

     
     },
    0x87b1 => {
     log("GeoTiffAsciiParams ");

         print_data(&data)

     
     },
    0x87be => {
     log("JBIGOptions ");

         print_data(&data)

     
     },
    0x8822 => {
     log("ExposureProgram ");

         print_data(&data)

     
     },
    0x8824 => {
     log("SpectralSensitivity ");

         print_data(&data)

     
     },
    0x8825 => {
     log("GPSInfo ");

         print_data(&data)

     
     },
    0x8827 => {
     log("ISO ");

         print_data(&data)

     
     },
    0x8828 => {
     log("Opto-ElectricConvFactor ");

         print_data(&data)

     
     },
    0x8829 => {
     log("Interlace ");

         print_data(&data)

     
     },
    0x882a => {
     log("TimeZoneOffset ");

         print_data(&data)

     
     },
    0x882b => {
     log("SelfTimerMode ");

         print_data(&data)

     
     },
    0x8830 => {
     log("SensitivityType ");

         print_data(&data)

     
     },
    0x8831 => {
     log("StandardOutputSensitivity ");

         print_data(&data)

     
     },
    0x8832 => {
     log("RecommendedExposureIndex ");

         print_data(&data)

     
     },
    0x8833 => {
     log("ISOSpeed ");

         print_data(&data)

     
     },
    0x8834 => {
     log("ISOSpeedLatitudeyyy ");

         print_data(&data)

     
     },
    0x8835 => {
     log("ISOSpeedLatitudezzz ");

         print_data(&data)

     
     },
    0x885c => {
     log("FaxRecvParams ");

         print_data(&data)

     
     },
    0x885d => {
     log("FaxSubAddress ");

         print_data(&data)

     
     },
    0x885e => {
     log("FaxRecvTime ");

         print_data(&data)

     
     },
    0x8871 => {
     log("FedexEDR ");

         print_data(&data)

     
     },
    0x888a => {
     log("LeafSubIFD ");

         print_data(&data)

     
     },
    0x9000 => {
     log("ExifVersion ");

         print_data(&data)

     
     },
    0x9003 => {
     log("DateTimeOriginal ");

         print_data(&data)

     
     },
    0x9004 => {
     log("CreateDate ");

         print_data(&data)

     
     },
    0x9009 => {
     log("GooglePlusUploadCode ");

         print_data(&data)

     
     },
    0x9010 => {
     log("OffsetTime ");

         print_data(&data)

     
     },
    0x9011 => {
     log("OffsetTimeOriginal ");

         print_data(&data)

     
     },
    0x9012 => {
     log("OffsetTimeDigitized ");

         print_data(&data)

     
     },
    0x9101 => {
     log("ComponentsConfiguration ");

         print_data(&data)

     
     },
    0x9102 => {
     log("CompressedBitsPerPixel ");

         print_data(&data)

     
     },
    0x9201 => {
     log("ShutterSpeedValue ");

         print_data(&data)

     
     },
    0x9202 => {
     log("ApertureValue ");

         print_data(&data)

     
     },
    0x9203 => {
     log("BrightnessValue ");

         print_data(&data)

     
     },
    0x9204 => {
     log("ExposureCompensation ");

         print_data(&data)

     
     },
    0x9205 => {
     log("MaxApertureValue ");

         print_data(&data)

     
     },
    0x9206 => {
     log("SubjectDistance ");

         print_data(&data)

     
     },
    0x9207 => {
     log("MeteringMode ");

         print_data(&data)

     
     },
    0x9208 => {
     log("LightSource ");

         print_data(&data)

     
     },
    0x9209 => {
     log("Flash ");

         print_data(&data)

     
     },
    0x920a => {
     log("FocalLength ");

         print_data(&data)

     
     },
    0x920b => {
     log("FlashEnergy ");

         print_data(&data)

     
     },
    0x920c => {
     log("SpatialFrequencyResponse ");

         print_data(&data)

     
     },
    0x920d => {
     log("Noise ");

         print_data(&data)

     
     },
    0x920e => {
     log("FocalPlaneXResolution ");

         print_data(&data)

     
     },
    0x920f => {
     log("FocalPlaneYResolution ");

         print_data(&data)

     
     },
    0x9210 => {
     log("FocalPlaneResolutionUnit ");

         print_data(&data)

     
     },
    0x9211 => {
     log("ImageNumber ");

         print_data(&data)

     
     },
    0x9212 => {
     log("SecurityClassification ");

         print_data(&data)

     
     },
    0x9213 => {
     log("ImageHistory ");

         print_data(&data)

     
     },
    0x9214 => {
     log("SubjectArea ");

         print_data(&data)

     
     },
    0x9215 => {
     log("ExposureIndex ");

         print_data(&data)

     
     },
    0x9216 => {
     log("TIFF-EPStandardID ");

         print_data(&data)

     
     },
    0x9217 => {
     log("SensingMethod ");

         print_data(&data)

     
     },
    0x923a => {
     log("CIP3DataFile ");

         print_data(&data)

     
     },
    0x923b => {
     log("CIP3Sheet ");

         print_data(&data)

     
     },
    0x923c => {
     log("CIP3Side ");

         print_data(&data)

     
     },
    0x923f => {
     log("StoNits ");

         print_data(&data)

     
     },
    0x927c => {
        log("MakerNoteApple ");
        match data {
            DataPack::Undef(d) => {

            let string = read_string(d, 0, d.len());
            log(&string)
            },
            _ => {}
        }

         print_data(&data)

     
     },
    0x9286 => {
     log("UserComment ");

         print_data(&data)

     
     },
    0x9290 => {
     log("SubSecTime ");

         print_data(&data)

     
     },
    0x9291 => {
     log("SubSecTimeOriginal ");

         print_data(&data)

     
     },
    0x9292 => {
     log("SubSecTimeDigitized ");

         print_data(&data)

     
     },
    0x932f => {
     log("MSDocumentText ");

         print_data(&data)

     
     },
    0x9330 => {
     log("MSPropertySetStorage ");

         print_data(&data)

     
     },
    0x9331 => {
     log("MSDocumentTextPosition ");

         print_data(&data)

     
     },
    0x935c => {
     log("ImageSourceData ");

         print_data(&data)

     
     },
    0x9400 => {
     log("AmbientTemperature ");

         print_data(&data)

     
     },
    0x9401 => {
     log("Humidity ");

         print_data(&data)

     
     },
    0x9402 => {
     log("Pressure ");

         print_data(&data)

     
     },
    0x9403 => {
     log("WaterDepth ");

         print_data(&data)

     
     },
    0x9404 => {
     log("Acceleration ");

         print_data(&data)

     
     },
    0x9405 => {
     log("CameraElevationAngle ");

         print_data(&data)

     
     },
    0x9c9b => {
     log("XPTitle ");

         print_data(&data)

     
     },
    0x9c9c => {
     log("XPComment ");

         print_data(&data)

     
     },
    0x9c9d => {
     log("XPAuthor ");

         print_data(&data)

     
     },
    0x9c9e => {
     log("XPKeywords ");

         print_data(&data)

     
     },
    0x9c9f => {
     log("XPSubject ");

         print_data(&data)

     
     },
    0xa000 => {
     log("FlashpixVersion ");

         print_data(&data)

     
     },
    0xa001 => {
     log("ColorSpace ");

         print_data(&data)

     
     },
    0xa002 => {
     log("ExifImageWidth ");

         print_data(&data)

     
     },
    0xa003 => {
     log("ExifImageHeight ");

         print_data(&data)

     
     },
    0xa004 => {
     log("RelatedSoundFile ");

         print_data(&data)

     
     },
    0xa005 => {
     log("InteropOffset ");

         print_data(&data)

     
     },
    0xa010 => {
     log("SamsungRawPointersOffset ");

         print_data(&data)

     
     },
    0xa011 => {
     log("SamsungRawPointersLength ");

         print_data(&data)

     
     },
    0xa101 => {
     log("SamsungRawByteOrder ");

         print_data(&data)

     
     },
    0xa102 => {
     log("SamsungRawUnknown? ");

         print_data(&data)

     
     },
    0xa20b => {
     log("FlashEnergy ");

         print_data(&data)

     
     },
    0xa20c => {
     log("SpatialFrequencyResponse ");

         print_data(&data)

     
     },
    0xa20d => {
     log("Noise ");

         print_data(&data)

     
     },
    0xa20e => {
     log("FocalPlaneXResolution ");

         print_data(&data)

     
     },
    0xa20f => {
     log("FocalPlaneYResolution ");

         print_data(&data)

     
     },
    0xa210 => {
     log("FocalPlaneResolutionUnit ");

         print_data(&data)

     
     },
    0xa211 => {
     log("ImageNumber ");

         print_data(&data)

     
     },
    0xa212 => {
     log("SecurityClassification ");

         print_data(&data)

     
     },
    0xa213 => {
     log("ImageHistory ");

         print_data(&data)

     
     },
    0xa214 => {
     log("SubjectLocation ");

         print_data(&data)

     
     },
    0xa215 => {
     log("ExposureIndex ");

         print_data(&data)

     
     },
    0xa216 => {
     log("TIFF-EPStandardID ");

         print_data(&data)

     
     },
    0xa217 => {
     log("SensingMethod ");

         print_data(&data)

     
     },
    0xa300 => {
     log("FileSource ");

         print_data(&data)

     
     },
    0xa301 => {
     log("SceneType ");

         print_data(&data)

     
     },
    0xa302 => {
     log("CFAPattern ");

         print_data(&data)

     
     },
    0xa401 => {
     log("CustomRendered ");

         print_data(&data)

     
     },
    0xa402 => {
     log("ExposureMode ");

         print_data(&data)

     
     },
    0xa403 => {
     log("WhiteBalance ");

         print_data(&data)

     
     },
    0xa404 => {
     log("DigitalZoomRatio ");

         print_data(&data)

     
     },
    0xa405 => {
     log("FocalLengthIn35mmFormat ");

         print_data(&data)

     
     },
    0xa406 => {
     log("SceneCaptureType ");

         print_data(&data)

     
     },
    0xa407 => {
     log("GainControl ");

         print_data(&data)

     
     },
    0xa408 => {
     log("Contrast ");

         print_data(&data)

     
     },
    0xa409 => {
     log("Saturation ");

         print_data(&data)

     
     },
    0xa40a => {
     log("Sharpness ");

         print_data(&data)

     
     },
    0xa40b => {
     log("DeviceSettingDescription ");

         print_data(&data)

     
     },
    0xa40c => {
     log("SubjectDistanceRange ");

         print_data(&data)

     
     },
    0xa420 => {
     log("ImageUniqueID ");

         print_data(&data)

     
     },
    0xa430 => {
     log("OwnerName ");

         print_data(&data)

     
     },
    0xa431 => {
     log("SerialNumber ");

         print_data(&data)

     
     },
    0xa432 => {
     log("LensInfo ");

         print_data(&data)

     
     },
    0xa433 => {
     log("LensMake ");

         print_data(&data)

     
     },
    0xa434 => {
     log("LensModel ");

         print_data(&data)

     
     },
    0xa435 => {
     log("LensSerialNumber ");

         print_data(&data)

     
     },
    0xa460 => {
     log("CompositeImage ");

         print_data(&data)

     
     },
    0xa461 => {
     log("CompositeImageCount ");

         print_data(&data)

     
     },
    0xa462 => {
     log("CompositeImageExposureTimes ");

         print_data(&data)

     
     },
    0xa480 => {
     log("GDALMetadata ");

         print_data(&data)

     
     },
    0xa481 => {
     log("GDALNoData ");

         print_data(&data)

     
     },
    0xa500 => {
     log("Gamma ");

         print_data(&data)

     
     },
    0xafc0 => {
     log("ExpandSoftware ");

         print_data(&data)

     
     },
    0xafc1 => {
     log("ExpandLens ");

         print_data(&data)

     
     },
    0xafc2 => {
     log("ExpandFilm ");

         print_data(&data)

     
     },
    0xafc3 => {
     log("ExpandFilterLens ");

         print_data(&data)

     
     },
    0xafc4 => {
     log("ExpandScanner ");

         print_data(&data)

     
     },
    0xafc5 => {
     log("ExpandFlashLamp ");

         print_data(&data)

     
     },
    0xb4c3 => {
     log("HasselbladRawImage ");

         print_data(&data)

     
     },
    0xbc01 => {
     log("PixelFormat ");

         print_data(&data)

     
     },
    0xbc02 => {
     log("Transformation ");

         print_data(&data)

     
     },
    0xbc03 => {
     log("Uncompressed ");

         print_data(&data)

     
     },
    0xbc04 => {
     log("ImageType ");

         print_data(&data)

     
     },
    0xbc80 => {
     log("ImageWidth ");

         print_data(&data)

     
     },
    0xbc81 => {
     log("ImageHeight ");

         print_data(&data)

     
     },
    0xbc82 => {
     log("WidthResolution ");

         print_data(&data)

     
     },
    0xbc83 => {
     log("HeightResolution ");

         print_data(&data)

     
     },
    0xbcc0 => {
     log("ImageOffset ");

         print_data(&data)

     
     },
    0xbcc1 => {
     log("ImageByteCount ");

         print_data(&data)

     
     },
    0xbcc2 => {
     log("AlphaOffset ");

         print_data(&data)

     
     },
    0xbcc3 => {
     log("AlphaByteCount ");

         print_data(&data)

     
     },
    0xbcc4 => {
     log("ImageDataDiscard ");

         print_data(&data)

     
     },
    0xbcc5 => {
     log("AlphaDataDiscard ");

         print_data(&data)

     
     },
    0xc427 => {
     log("OceScanjobDesc ");

         print_data(&data)

     
     },
    0xc428 => {
     log("OceApplicationSelector ");

         print_data(&data)

     
     },
    0xc429 => {
     log("OceIDNumber ");

         print_data(&data)

     
     },
    0xc42a => {
     log("OceImageLogic ");

         print_data(&data)

     
     },
    0xc44f => {
     log("Annotations ");

         print_data(&data)

     
     },
    0xc4a5 => {
     log("PrintIM ");

         print_data(&data)

     
     },
    0xc51b => {
     log("HasselbladExif ");

         print_data(&data)

     
     },
    0xc573 => {
     log("OriginalFileName ");

         print_data(&data)

     
     },
    0xc580 => {
     log("USPTOOriginalContentType ");

         print_data(&data)

     
     },
    0xc5e0 => {
     log("CR2CFAPattern ");

         print_data(&data)

     
     },
    0xc612 => {
     log("DNGVersion ");

         print_data(&data)

     
     },
    0xc613 => {
     log("DNGBackwardVersion ");

         print_data(&data)

     
     },
    0xc614 => {
     log("UniqueCameraModel ");

         print_data(&data)

     
     },
    0xc615 => {
     log("LocalizedCameraModel ");

         print_data(&data)

     
     },
    0xc616 => {
     log("CFAPlaneColor ");

         print_data(&data)

     
     },
    0xc617 => {
     log("CFALayout ");

         print_data(&data)

     
     },
    0xc618 => {
     log("LinearizationTable ");

         print_data(&data)

     
     },
    0xc619 => {
     log("BlackLevelRepeatDim ");

         print_data(&data)

     
     },
    0xc61a => {
     log("BlackLevel ");

         print_data(&data)

     
     },
    0xc61b => {
     log("BlackLevelDeltaH ");

         print_data(&data)

     
     },
    0xc61c => {
     log("BlackLevelDeltaV ");

         print_data(&data)

     
     },
    0xc61d => {
     log("WhiteLevel ");

         print_data(&data)

     
     },
    0xc61e => {
     log("DefaultScale ");

         print_data(&data)

     
     },
    0xc61f => {
     log("DefaultCropOrigin ");

         print_data(&data)

     
     },
    0xc620 => {
     log("DefaultCropSize ");

         print_data(&data)

     
     },
    0xc621 => {
     log("ColorMatrix1 ");

         print_data(&data)

     
     },
    0xc622 => {
     log("ColorMatrix2 ");

         print_data(&data)

     
     },
    0xc623 => {
     log("CameraCalibration1 ");

         print_data(&data)

     
     },
    0xc624 => {
     log("CameraCalibration2 ");

         print_data(&data)

     
     },
    0xc625 => {
     log("ReductionMatrix1 ");

         print_data(&data)

     
     },
    0xc626 => {
     log("ReductionMatrix2 ");

         print_data(&data)

     
     },
    0xc627 => {
     log("AnalogBalance ");

         print_data(&data)

     
     },
    0xc628 => {
     log("AsShotNeutral ");

         print_data(&data)

     
     },
    0xc629 => {
     log("AsShotWhiteXY ");

         print_data(&data)

     
     },
    0xc62a => {
     log("BaselineExposure ");

         print_data(&data)

     
     },
    0xc62b => {
     log("BaselineNoise ");

         print_data(&data)

     
     },
    0xc62c => {
     log("BaselineSharpness ");

         print_data(&data)

     
     },
    0xc62d => {
     log("BayerGreenSplit ");

         print_data(&data)

     
     },
    0xc62e => {
     log("LinearResponseLimit ");

         print_data(&data)

     
     },
    0xc62f => {
     log("CameraSerialNumber ");

         print_data(&data)

     
     },
    0xc630 => {
     log("DNGLensInfo ");

         print_data(&data)

     
     },
    0xc631 => {
     log("ChromaBlurRadius ");

         print_data(&data)

     
     },
    0xc632 => {
     log("AntiAliasStrength ");

         print_data(&data)

     
     },
    0xc633 => {
     log("ShadowScale ");

         print_data(&data)

     
     },
    0xc634 => {
     log("SR2Private ");

         print_data(&data)

     
     },
    0xc635 => {
     log("MakerNoteSafety ");

         print_data(&data)

     
     },
    0xc640 => {
     log("RawImageSegmentation ");

         print_data(&data)

     
     },
    0xc65a => {
     log("CalibrationIlluminant1 ");

         print_data(&data)

     
     },
    0xc65b => {
     log("CalibrationIlluminant2 ");

         print_data(&data)

     
     },
    0xc65c => {
     log("BestQualityScale ");

         print_data(&data)

     
     },
    0xc65d => {
     log("RawDataUniqueID ");

         print_data(&data)

     
     },
    0xc660 => {
     log("AliasLayerMetadata ");

         print_data(&data)

     
     },
    0xc68b => {
     log("OriginalRawFileName ");

         print_data(&data)

     
     },
    0xc68c => {
     log("OriginalRawFileData ");

         print_data(&data)

     
     },
    0xc68d => {
     log("ActiveArea ");

         print_data(&data)

     
     },
    0xc68e => {
     log("MaskedAreas ");

         print_data(&data)

     
     },
    0xc68f => {
     log("AsShotICCProfile ");

         print_data(&data)

     
     },
    0xc690 => {
     log("AsShotPreProfileMatrix ");

         print_data(&data)

     
     },
    0xc691 => {
     log("CurrentICCProfile ");

         print_data(&data)

     
     },
    0xc692 => {
     log("CurrentPreProfileMatrix ");

         print_data(&data)

     
     },
    0xc6bf => {
     log("ColorimetricReference ");

         print_data(&data)

     
     },
    0xc6c5 => {
     log("SRawType ");

         print_data(&data)

     
     },
    0xc6d2 => {
     log("PanasonicTitle ");

         print_data(&data)

     
     },
    0xc6d3 => {
     log("PanasonicTitle2 ");

         print_data(&data)

     
     },
    0xc6f3 => {
     log("CameraCalibrationSig ");

         print_data(&data)

     
     },
    0xc6f4 => {
     log("ProfileCalibrationSig ");

         print_data(&data)

     
     },
    0xc6f5 => {
     log("ProfileIFD ");

         print_data(&data)

     
     },
    0xc6f6 => {
     log("AsShotProfileName ");

         print_data(&data)

     
     },
    0xc6f7 => {
     log("NoiseReductionApplied ");

         print_data(&data)

     
     },
    0xc6f8 => {
     log("ProfileName ");

         print_data(&data)

     
     },
    0xc6f9 => {
     log("ProfileHueSatMapDims ");

         print_data(&data)

     
     },
    0xc6fa => {
     log("ProfileHueSatMapData1 ");

         print_data(&data)

     
     },
    0xc6fb => {
     log("ProfileHueSatMapData2 ");

         print_data(&data)

     
     },
    0xc6fc => {
     log("ProfileToneCurve ");

         print_data(&data)

     
     },
    0xc6fd => {
     log("ProfileEmbedPolicy ");

         print_data(&data)

     
     },
    0xc6fe => {
     log("ProfileCopyright ");

         print_data(&data)

     
     },
    0xc714 => {
     log("ForwardMatrix1 ");

         print_data(&data)

     
     },
    0xc715 => {
     log("ForwardMatrix2 ");

         print_data(&data)

     
     },
    0xc716 => {
     log("PreviewApplicationName ");

         print_data(&data)

     
     },
    0xc717 => {
     log("PreviewApplicationVersion ");

         print_data(&data)

     
     },
    0xc718 => {
     log("PreviewSettingsName ");

         print_data(&data)

     
     },
    0xc719 => {
     log("PreviewSettingsDigest ");

         print_data(&data)

     
     },
    0xc71a => {
     log("PreviewColorSpace ");

         print_data(&data)

     
     },
    0xc71b => {
     log("PreviewDateTime ");

         print_data(&data)

     
     },
    0xc71c => {
     log("RawImageDigest ");

         print_data(&data)

     
     },
    0xc71d => {
     log("OriginalRawFileDigest ");

         print_data(&data)

     
     },
    0xc71e => {
     log("SubTileBlockSize ");

         print_data(&data)

     
     },
    0xc71f => {
     log("RowInterleaveFactor ");

         print_data(&data)

     
     },
    0xc725 => {
     log("ProfileLookTableDims ");

         print_data(&data)

     
     },
    0xc726 => {
     log("ProfileLookTableData ");

         print_data(&data)

     
     },
    0xc740 => {
     log("OpcodeList1 ");

         print_data(&data)

     
     },
    0xc741 => {
     log("OpcodeList2 ");

         print_data(&data)

     
     },
    0xc74e => {
     log("OpcodeList3 ");

         print_data(&data)

     
     },
    0xc761 => {
     log("NoiseProfile ");

         print_data(&data)

     
     },
    0xc763 => {
     log("TimeCodes ");

         print_data(&data)

     
     },
    0xc764 => {
     log("FrameRate ");

         print_data(&data)

     
     },
    0xc772 => {
     log("TStop ");

         print_data(&data)

     
     },
    0xc789 => {
     log("ReelName ");

         print_data(&data)

     
     },
    0xc791 => {
     log("OriginalDefaultFinalSize ");

         print_data(&data)

     
     },
    0xc792 => {
     log("OriginalBestQualitySize ");

         print_data(&data)

     
     },
    0xc793 => {
     log("OriginalDefaultCropSize ");

         print_data(&data)

     
     },
    0xc7a1 => {
     log("CameraLabel ");

         print_data(&data)

     
     },
    0xc7a3 => {
     log("ProfileHueSatMapEncoding ");

         print_data(&data)

     
     },
    0xc7a4 => {
     log("ProfileLookTableEncoding ");

         print_data(&data)

     
     },
    0xc7a5 => {
     log("BaselineExposureOffset ");

         print_data(&data)

     
     },
    0xc7a6 => {
     log("DefaultBlackRender ");

         print_data(&data)

     
     },
    0xc7a7 => {
     log("NewRawImageDigest ");

         print_data(&data)

     
     },
    0xc7a8 => {
     log("RawToPreviewGain ");

         print_data(&data)

     
     },
    0xc7aa => {
     log("CacheVersion ");

         print_data(&data)

     
     },
    0xc7b5 => {
     log("DefaultUserCrop ");

         print_data(&data)

     
     },
    0xc7d5 => {
     log("NikonNEFInfo ");

         print_data(&data)

     
     },
    0xc7e9 => {
     log("DepthFormat ");

         print_data(&data)

     
     },
    0xc7ea => {
     log("DepthNear ");

         print_data(&data)

     
     },
    0xc7eb => {
     log("DepthFar ");

         print_data(&data)

     
     },
    0xc7ec => {
     log("DepthUnits ");

         print_data(&data)

     
     },
    0xc7ed => {
     log("DepthMeasureType ");

         print_data(&data)

     
     },
    0xc7ee => {
     log("EnhanceParams ");

         print_data(&data)

     
     },
    0xcd2d => {
     log("ProfileGainTableMap ");

         print_data(&data)

     
     },
    0xcd2e => {
     log("SemanticName ");

         print_data(&data)

     
     },
    0xcd30 => {
     log("SemanticInstanceIFD ");

         print_data(&data)

     
     },
    0xcd31 => {
     log("CalibrationIlluminant3 ");

         print_data(&data)

     
     },
    0xcd32 => {
     log("CameraCalibration3 ");

         print_data(&data)

     
     },
    0xcd33 => {
     log("ColorMatrix3 ");

         print_data(&data)

     
     },
    0xcd34 => {
     log("ForwardMatrix3 ");

         print_data(&data)

     
     },
    0xcd35 => {
     log("IlluminantData1 ");

         print_data(&data)

     
     },
    0xcd36 => {
     log("IlluminantData2 ");

         print_data(&data)

     
     },
    0xcd37 => {
     log("IlluminantData3 ");

         print_data(&data)

     
     },
    0xcd38 => {
     log("MaskSubArea ");

         print_data(&data)

     
     },
    0xcd39 => {
     log("ProfileHueSatMapData3 ");

         print_data(&data)

     
     },
    0xcd3a => {
     log("ReductionMatrix3 ");

         print_data(&data)

     
     },
    0xcd3b => {
     log("RGBTables ");

         print_data(&data)

     
     },
    0xea1c => {
     log("Padding ");

         print_data(&data)

     
     },
    0xea1d => {
     log("OffsetSchema ");

         print_data(&data)

     
     },
    0xfde8 => {
     log("OwnerName ");

         print_data(&data)

     
     },
    0xfde9 => {
     log("SerialNumber ");

         print_data(&data)

     
     },
    0xfdea => {
     log("Lens ");

         print_data(&data)

     
     },
    0xfe00 => {
     log("KDC_IFD ");

         print_data(&data)

     
     },
    0xfe4c => {
     log("RawFile ");

         print_data(&data)

     
     },
    0xfe4d => {
     log("Converter ");

         print_data(&data)

     
     },
    0xfe4e => {
     log("WhiteBalance ");

         print_data(&data)

     
     },
    0xfe51 => {
     log("Exposure ");

         print_data(&data)

     
     },
    0xfe52 => {
     log("Shadows ");

         print_data(&data)

     
     },
    0xfe53 => {
     log("Brightness ");

         print_data(&data)

     
     },
    0xfe54 => {
     log("Contrast ");

         print_data(&data)

     
     },
    0xfe55 => {
     log("Saturation ");

         print_data(&data)

     
     },
    0xfe56 => {
     log("Sharpness ");

         print_data(&data)

     
     },
    0xfe57 => {
     log("Smoothness ");

         print_data(&data)

     
     },
    0xfe58 => {
     log("MoireFilter ");

         print_data(&data)

     
     },
    _ => {
       print_data(&data)       
    },
    }
}
