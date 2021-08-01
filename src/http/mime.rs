pub enum MimeType {
    TextCss,
    TextHtml,
    TextJavaScript,
    TextPlain,
    ImageBmp,
    ImageGif,
    ImageJpeg,
    ImagePng,
    ImageWebp,
    ImageSvg,
    ImageIcon,
    ApplicationOctetStream,
    ApplicationJson,
    ApplicationPdf,
    ApplicationZip,
    VideoMp4,
    VideoOgg,
    VideoWebm,
    FontTtf,
    FontOtf,
    FontWoff,
    FontWoff2,
}

impl MimeType {
    pub fn from_extension(extension: &str) -> Self {
        match extension {
            ".css" => MimeType::TextCss,
            ".html" => MimeType::TextHtml,
            ".htm" => MimeType::TextHtml,
            ".js" => MimeType::TextJavaScript,
            ".txt" => MimeType::TextPlain,
            ".bmp" => MimeType::ImageBmp,
            ".gif" => MimeType::ImageGif,
            ".jpeg" => MimeType::ImageJpeg,
            ".jpg" => MimeType::ImageJpeg,
            ".png" => MimeType::ImagePng,
            ".webp" => MimeType::ImageWebp,
            ".svg" => MimeType::ImageSvg,
            ".ico" => MimeType::ImageIcon,
            ".json" => MimeType::ApplicationJson,
            ".pdf" => MimeType::ApplicationPdf,
            ".zip" => MimeType::ApplicationZip,
            ".mp4" => MimeType::VideoMp4,
            ".ogv" => MimeType::VideoOgg,
            ".webm" => MimeType::VideoWebm,
            ".ttf" => MimeType::FontTtf,
            ".otf" => MimeType::FontOtf,
            ".woff" => MimeType::FontWoff,
            ".woff2" => MimeType::FontWoff2,
            _ => MimeType::ApplicationOctetStream,
        }
    }
}

impl Into<&str> for MimeType {
    fn into(self) -> &'static str {
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
    }
}
