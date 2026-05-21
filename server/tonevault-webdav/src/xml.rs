pub struct MultiStatus {
    pub responses: Vec<DavResponse>,
}

pub struct DavResponse {
    pub href: String,
    pub propstat: Vec<PropStat>,
}

pub struct PropStat {
    pub prop: DavProp,
    pub status: String,
}

pub struct DavProp {
    pub resource_type: Option<ResourceType>,
    pub content_type: Option<String>,
    pub content_length: Option<u64>,
    pub display_name: Option<String>,
    pub last_modified: Option<String>,
    pub creation_date: Option<String>,
    pub etag: Option<String>,
}

pub enum ResourceType {
    Collection,
}

impl MultiStatus {
    pub fn to_xml(&self) -> String {
        let mut xml = String::from("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n");
        xml.push_str("<D:multistatus xmlns:D=\"DAV:\">\n");

        for resp in &self.responses {
            xml.push_str("  <D:response>\n");
            xml.push_str(&format!("    <D:href>{}</D:href>\n", xml_escape(&resp.href)));

            for ps in &resp.propstat {
                xml.push_str("    <D:propstat>\n");
                xml.push_str("      <D:prop>\n");

                match &ps.prop.resource_type {
                    Some(ResourceType::Collection) => {
                        xml.push_str("        <D:resourcetype><D:collection/></D:resourcetype>\n");
                    }
                    None => {
                        xml.push_str("        <D:resourcetype/>\n");
                    }
                }

                if let Some(ct) = &ps.prop.content_type {
                    xml.push_str(&format!("        <D:getcontenttype>{}</D:getcontenttype>\n", xml_escape(ct)));
                }
                if let Some(cl) = ps.prop.content_length {
                    xml.push_str(&format!("        <D:getcontentlength>{}</D:getcontentlength>\n", cl));
                }
                if let Some(dn) = &ps.prop.display_name {
                    xml.push_str(&format!("        <D:displayname>{}</D:displayname>\n", xml_escape(dn)));
                }
                if let Some(lm) = &ps.prop.last_modified {
                    xml.push_str(&format!("        <D:getlastmodified>{}</D:getlastmodified>\n", xml_escape(lm)));
                }
                if let Some(cd) = &ps.prop.creation_date {
                    xml.push_str(&format!("        <D:creationdate>{}</D:creationdate>\n", xml_escape(cd)));
                }
                if let Some(et) = &ps.prop.etag {
                    xml.push_str(&format!("        <D:getetag>{}</D:getetag>\n", xml_escape(et)));
                }

                xml.push_str("      </D:prop>\n");
                xml.push_str(&format!("      <D:status>{}</D:status>\n", ps.status));
                xml.push_str("    </D:propstat>\n");
            }

            xml.push_str("  </D:response>\n");
        }

        xml.push_str("</D:multistatus>");
        xml
    }
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
