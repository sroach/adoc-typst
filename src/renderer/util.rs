/// Decode common HTML entities that may appear in text content, titles, and metadata.
pub fn decode_html_entities(text: &str) -> String {
    text.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&#39;", "'")
        .replace("&#8217;", "\u{2019}")  // right single quotation mark
        .replace("&#8216;", "\u{2018}")  // left single quotation mark
        .replace("&#8220;", "\u{201C}")  // left double quotation mark
        .replace("&#8221;", "\u{201D}")  // right double quotation mark
        .replace("&#8211;", "\u{2013}")  // en dash
        .replace("&#8212;", "\u{2014}")  // em dash
        .replace("&#8230;", "\u{2026}")  // horizontal ellipsis
        .replace("&#8203;", "\u{200B}")  // zero-width space
        .replace("&#8482;", "™")          // trademark
        .replace("&#8242;", "\u{2032}")  // prime
        .replace("&#x20ac;", "€")         // euro sign
        .replace("&nbsp;", "\u{00A0}")   // non-breaking space
        .replace("&ndash;", "\u{2013}")  // en dash
        .replace("&mdash;", "\u{2014}")  // em dash
        .replace("&hellip;", "\u{2026}") // horizontal ellipsis
}
