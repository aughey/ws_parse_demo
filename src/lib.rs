use std::io::{BufRead, BufReader};

use anyhow::Result;

pub struct Data {
    pub data: Vec<u8>,
    pub src_port: u16,
    pub dst_port: u16,
    pub src_addr: String,
    pub dst_addr: String,
    pub time: f64,
}

pub fn read_file(filename: &str) -> Result<impl Iterator<Item = Data>> {
    let file = std::fs::File::open(filename)?;
    let file = BufReader::new(file);

    Ok(read_buf(file))
}

struct SearchDef {
    start: &'static str,
    value: Option<String>,
}

pub fn read_buf(buffer: impl BufRead) -> impl Iterator<Item = Data> {
    // We treat this as a state machine of sorts.
    let mut values = [
        SearchDef {
            start: r#""udp.srcport": ""#,
            value: None,
        },
        SearchDef {
            start: r#""udp.dstport": ""#,
            value: None,
        },
        SearchDef {
            start: r#""ip.src": ""#,
            value: None,
        },
        SearchDef {
            start: r#""ip.dst": ""#,
            value: None,
        },
        SearchDef {
            start: r#""frame.time_relative": ""#,
            value: None,
        },
    ];

    let mut lines = buffer.lines();

    std::iter::from_fn(move || {
        while let Some(Ok(line)) = lines.next() {
            // Treat frame and udp.payload specially
            if line.contains(r#""frame": {"#) {
                for v in values.iter_mut() {
                    v.value = None;
                }
                continue;
            }

            const PAYLOAD_START: &str = r#""udp.payload": ""#;
            if let Some(index) = line.find(PAYLOAD_START) {
                let value = &line[index + PAYLOAD_START.len()..];
                if let Some(closing_quote) = value.find('"') {
                    let data = &value[..closing_quote];

                    let data = data
                        .split(':')
                        .map(|x| u8::from_str_radix(x, 16))
                        .collect::<Result<Vec<_>, _>>();

                    if let Ok(data) = data {
                        if values.iter().all(|v| v.value.is_some()) {
                            return Some(Data {
                                data: data,
                                src_port: values[0].value.take().unwrap().parse().unwrap(),
                                dst_port: values[1].value.take().unwrap().parse().unwrap(),
                                src_addr: values[2].value.take().unwrap(),
                                dst_addr: values[3].value.take().unwrap(),
                                time: values[4].value.take().unwrap().parse().unwrap(),
                            });
                        }
                    }
                }
            }

            for v in values.iter_mut() {
                if let Some(index) = line.find(v.start) {
                    let value = &line[index + v.start.len()..];
                    if let Some(closing_quote) = value.find('"') {
                        v.value = Some(value[..closing_quote].to_string());
                    }
                }
            }
        }
        None
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // include "osc.json"
    const TEST_DATA: &str = include_str!("../osc.json");

    #[test]
    fn test_read_file() {
        let data = read_buf(BufReader::new(TEST_DATA.as_bytes()));
        let data = data.collect::<Vec<_>>();
        assert_eq!(data.len(), 282);
        let first = data.first().unwrap();
        assert_eq!(first.src_port, 10023);
        assert_eq!(first.dst_port, 50576);
        assert_eq!(first.src_addr, "10.0.0.50");
        assert_eq!(first.dst_addr, "10.10.0.6");
        assert_eq!(first.time, 1.796723000);
        assert_eq!(first.data.len(), 52);
    }
}
