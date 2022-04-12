//! Provides functionality for handling MIME types.

/// Represents a MIME type as used in the `Content-Type` header.
#[derive(Clone, Copy)]
pub enum MimeType {
    /// The `text/css` MIME type.
    TextCss,
    /// The `text/html` MIME type.
    TextHtml,
    /// The `text/javascript` MIME type.
    TextJavaScript,
    /// The `text/plain` MIME type.
    TextPlain,
    /// The `image/bmp` MIME type.
    ImageBmp,
    /// The `image/gif` MIME type.
    ImageGif,
    /// The `image/jpeg` MIME type.
    ImageJpeg,
    /// The `image/png` MIME type.
    ImagePng,
    /// The `image/webp` MIME type.
    ImageWebp,
    /// The `image/svg+xml` MIME type.
    ImageSvg,
    /// The `image/vnd.microsoft.icon` MIME type.
    ImageIcon,
    /// The `application/octet-stream` MIME type.
    ApplicationOctetStream,
    /// The `application/json` MIME type.
    ApplicationJson,
    /// The `application/pdf` MIME type.
    ApplicationPdf,
    /// The `application/zip` MIME type.
    ApplicationZip,
    /// The `video/mp4` MIME type.
    VideoMp4,
    /// The `video/ogg` MIME type.
    VideoOgg,
    /// The `video/webm` MIME type.
    VideoWebm,
    /// The `font/ttf` MIME type.
    FontTtf,
    /// The `font/otf` MIME type.
    FontOtf,
    /// The `font/woff` MIME type.
    FontWoff,
    /// The `font/woff2` MIME type.
    FontWoff2,
}

impl MimeType {
    /// Converts from a file extension without the `.` to the enum variant.
    /// If the MIME type cannot be inferred from the extension, returns `MimeType::ApplicationOctetStream`.
    pub fn from_extension(extension: &str) -> Self {
        match extension {
            "css" => MimeType::TextCss,
            "html" => MimeType::TextHtml,
            "htm" => MimeType::TextHtml,
            "js" => MimeType::TextJavaScript,
            "txt" => MimeType::TextPlain,
            "bmp" => MimeType::ImageBmp,
            "gif" => MimeType::ImageGif,
            "jpeg" => MimeType::ImageJpeg,
            "jpg" => MimeType::ImageJpeg,
            "png" => MimeType::ImagePng,
            "webp" => MimeType::ImageWebp,
            "svg" => MimeType::ImageSvg,
            "ico" => MimeType::ImageIcon,
            "json" => MimeType::ApplicationJson,
            "pdf" => MimeType::ApplicationPdf,
            "zip" => MimeType::ApplicationZip,
            "mp4" => MimeType::VideoMp4,
            "ogv" => MimeType::VideoOgg,
            "webm" => MimeType::VideoWebm,
            "ttf" => MimeType::FontTtf,
            "otf" => MimeType::FontOtf,
            "woff" => MimeType::FontWoff,
            "woff2" => MimeType::FontWoff2,
            _ => MimeType::ApplicationOctetStream,
        }
    }
}

impl From<MimeType> for String {
    fn from(val: MimeType) -> Self {
        val.to_string()
    }
}

impl ToString for MimeType {
    fn to_string(&self) -> String {
        match self {
            MimeType::TextCss => "text/css",
            MimeType::TextHtml => "text/html",
            MimeType::TextJavaScript => "text/javascript",
            MimeType::TextPlain => "text/plain",
            MimeType::ImageBmp => "image/bmp",
            MimeType::ImageGif => "image/gif",
            MimeType::ImageJpeg => "image/jpeg",
            MimeType::ImagePng => "image/png",
            MimeType::ImageWebp => "image/webp",
            MimeType::ImageSvg => "image/svg+xml",
            MimeType::ImageIcon => "image/vnd.microsoft.icon",
            MimeType::ApplicationOctetStream => "application/octet-stream",
            MimeType::ApplicationJson => "application/json",
            MimeType::ApplicationPdf => "application/pdf",
            MimeType::ApplicationZip => "application/zip",
            MimeType::VideoMp4 => "video/mp4",
            MimeType::VideoOgg => "video/ogg",
            MimeType::VideoWebm => "video/webm",
            MimeType::FontTtf => "font/ttf",
            MimeType::FontOtf => "font/otf",
            MimeType::FontWoff => "font/woff",
            MimeType::FontWoff2 => "font/woff2",
        }
        .to_string()
    }
}
