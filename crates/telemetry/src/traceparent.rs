use std::fmt;

pub const TRACEPARENT_ENV: &str = "TRACEPARENT";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TraceParent {
    pub trace_id: [u8; 16],
    pub span_id: [u8; 8],
    pub sampled: bool,
}

impl TraceParent {
    pub fn parse(header: &str) -> Option<Self> {
        let parts: Vec<_> = header.split('-').collect();
        if parts.len() != 4
            || parts[0] != "00"
            || parts[1].len() != 32
            || parts[2].len() != 16
            || parts[3].len() != 2
            || header.bytes().any(|b| b.is_ascii_uppercase())
        {
            return None;
        }
        let trace_id: [u8; 16] = hex::decode(parts[1]).ok()?.try_into().ok()?;
        let span_id: [u8; 8] = hex::decode(parts[2]).ok()?.try_into().ok()?;
        let flags = u8::from_str_radix(parts[3], 16).ok()?;
        if trace_id == [0; 16] || span_id == [0; 8] {
            return None;
        }
        Some(Self {
            trace_id,
            span_id,
            sampled: flags % 2 == 1,
        })
    }
    pub fn child(self, span_id: [u8; 8]) -> Self {
        Self { span_id, ..self }
    }
    pub fn trace_id_hex(self) -> String {
        hex::encode(self.trace_id)
    }
    pub fn span_id_hex(self) -> String {
        hex::encode(self.span_id)
    }
}
impl fmt::Display for TraceParent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "00-{}-{}-{:02x}",
            self.trace_id_hex(),
            self.span_id_hex(),
            u8::from(self.sampled)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_traceparent_roundtrip_and_invalid_ids() {
        let header = "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01";
        let parent = TraceParent::parse(header).unwrap();
        assert_eq!(parent.to_string(), header);
        assert_eq!(parent.child([2; 8]).trace_id, parent.trace_id);
        for invalid in [
            "",
            "ff-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01",
            "00-00000000000000000000000000000000-b7ad6b7169203331-01",
            "00-0af7651916cd43dd8448eb211c80319c-0000000000000000-01",
        ] {
            assert!(TraceParent::parse(invalid).is_none());
        }
    }
}
