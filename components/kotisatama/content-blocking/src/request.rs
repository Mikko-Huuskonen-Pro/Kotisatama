//! Pyyntömalli ilman adblock-tyyppejä.

/// Resurssityyppi Servo/Katselin-tasolla (muunnetaan adapterissa).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceType {
    Document,
    Subdocument,
    Script,
    Stylesheet,
    Image,
    Font,
    XmlHttpRequest,
    Media,
    Websocket,
    Other,
}

impl ResourceType {
    /// adblock-rust CPT-merkkijono (`Request::new` -yhteensopiva).
    pub fn as_adblock_cpt(self) -> &'static str {
        match self {
            Self::Document => "document",
            Self::Subdocument => "subdocument",
            Self::Script => "script",
            Self::Stylesheet => "stylesheet",
            Self::Image => "image",
            Self::Font => "font",
            Self::XmlHttpRequest => "xmlhttprequest",
            Self::Media => "media",
            Self::Websocket => "websocket",
            Self::Other => "other",
        }
    }
}

/// Yksi tarkistettava verkkopyyntö.
#[derive(Debug, Clone, Copy)]
pub struct BlockingRequest<'a> {
    pub url: &'a str,
    pub source_url: &'a str,
    pub resource_type: ResourceType,
}
