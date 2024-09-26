#![allow(dead_code)]
#![allow(non_upper_case_globals)]
extern crate saphyr_parser;

use saphyr_parser::{Event, EventReceiver, Parser, TScalarStyle};

// These names match the names used in the C++ test suite.
#[allow(clippy::enum_variant_names)]
#[derive(Clone, PartialEq, PartialOrd, Debug)]
enum TestEvent {
    OnDocumentStart,
    OnDocumentEnd,
    OnSequenceStart,
    OnSequenceEnd,
    OnMapStart,
    OnMapEnd,
    OnScalar,
    OnAlias,
    OnNull,
}

struct YamlChecker {
    pub evs: Vec<TestEvent>,
}

impl EventReceiver for YamlChecker {
    fn on_event(&mut self, ev: Event) {
        let tev = match ev {
            Event::DocumentStart(_) => TestEvent::OnDocumentStart,
            Event::DocumentEnd => TestEvent::OnDocumentEnd,
            Event::SequenceStart(..) => TestEvent::OnSequenceStart,
            Event::SequenceEnd => TestEvent::OnSequenceEnd,
            Event::MappingStart(..) => TestEvent::OnMapStart,
            Event::MappingEnd => TestEvent::OnMapEnd,
            Event::Scalar(ref v, style, _, _) => {
                if v == "~" && style == TScalarStyle::Plain {
                    TestEvent::OnNull
                } else {
                    TestEvent::OnScalar
                }
            }
            Event::Alias(_) => TestEvent::OnAlias,
            _ => return, // ignore other events
        };
        self.evs.push(tev);
    }
}

fn str_to_test_events(docs: &str) -> Vec<TestEvent> {
    let mut p = YamlChecker { evs: Vec::new() };
    let mut parser = Parser::new_from_str(docs);
    parser.load(&mut p, true).unwrap();
    p.evs
}

macro_rules! assert_next {
    ($v:expr, $p:pat) => {
        match $v.next().unwrap() {
            $p => {}
            e => {
                panic!("unexpected event: {:?} (expected {:?})", e, stringify!($p));
            }
        }
    };
}

// auto generated from handler_spec_test.cpp
include!("specexamples.rs.inc");
include!("spec_test.rs.inc");

mod with_buffered_input {
    use super::{Parser, TestEvent, YamlChecker};

    fn str_to_test_events(docs: &str) -> Vec<TestEvent> {
        use saphyr_parser::BufferedInput;

        let mut p = YamlChecker { evs: Vec::new() };
        let input = BufferedInput::new(docs.chars());
        let mut parser = Parser::new(input);
        parser.load(&mut p, true).unwrap();
        p.evs
    }
    include!("specexamples.rs.inc");
    include!("spec_test.rs.inc");
}
