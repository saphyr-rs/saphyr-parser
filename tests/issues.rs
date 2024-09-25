use saphyr_parser::{Event, Parser, ScanError, TScalarStyle};

/// Run the parser through the string.
///
/// # Returns
/// This functions returns the events if parsing succeeds, the error the parser returned otherwise.
fn run_parser(input: &str) -> Result<Vec<Event>, ScanError> {
    let mut events = vec![];
    for x in Parser::new_from_str(input) {
        events.push(x?.0);
    }
    Ok(events)
}

#[test]
#[allow(clippy::too_many_lines)]
fn test_issue1() {
    // https://github.com/saphyr-rs/saphyr-parser/issues/1

    // Implicit mappings with a nested flow sequence prematurely end the mapping.
    //
    // [a: [42]]
    //        ^ This closing sequence character
    // ^ is interpreted as closing this sequence, where the implicit mapping starts.
    //
    // The current implementation does not allow for nested implicit mapping:
    //
    // [a: [b: c]]
    //      ^ this implicit mapping would be ignored
    let reference = r"
- a:
  - 42
";

    let expected = [
        Event::StreamStart,
        Event::DocumentStart(false),
        Event::SequenceStart(0, None),
        Event::MappingStart(0, None),
        Event::Scalar("a".to_string(), TScalarStyle::Plain, 0, None),
        Event::SequenceStart(0, None),
        Event::Scalar("42".to_string(), TScalarStyle::Plain, 0, None),
        Event::SequenceEnd,
        Event::MappingEnd,
        Event::SequenceEnd,
        Event::DocumentEnd,
        Event::StreamEnd,
    ];
    assert_eq!(run_parser(reference).unwrap(), expected);
    assert_eq!(run_parser("[{a: [42]}]").unwrap(), expected);
    assert_eq!(run_parser("[a: [42]]").unwrap(), expected);

    // Other test cases derived from the bug

    // Implicit mapping in a complex key.
    assert_eq!(
        run_parser("[foo: [bar]]: baz").unwrap(),
        [
            Event::StreamStart,
            Event::DocumentStart(false),
            Event::MappingStart(0, None),
            Event::SequenceStart(0, None),
            Event::MappingStart(0, None),
            Event::Scalar("foo".to_string(), TScalarStyle::Plain, 0, None),
            Event::SequenceStart(0, None),
            Event::Scalar("bar".to_string(), TScalarStyle::Plain, 0, None),
            Event::SequenceEnd,
            Event::MappingEnd,
            Event::SequenceEnd,
            Event::Scalar("baz".to_string(), TScalarStyle::Plain, 0, None),
            Event::MappingEnd,
            Event::DocumentEnd,
            Event::StreamEnd,
        ]
    );

    // Implicit mappings with implicit null keys
    assert_eq!(
        run_parser("[:]").unwrap(),
        [
            Event::StreamStart,
            Event::DocumentStart(false),
            Event::SequenceStart(0, None),
            Event::MappingStart(0, None),
            Event::Scalar("~".to_string(), TScalarStyle::Plain, 0, None),
            Event::Scalar("~".to_string(), TScalarStyle::Plain, 0, None),
            Event::MappingEnd,
            Event::SequenceEnd,
            Event::DocumentEnd,
            Event::StreamEnd,
        ]
    );

    // Nested implicit mappings with implicit null keys
    assert_eq!(
        run_parser("[: [:]]").unwrap(),
        [
            Event::StreamStart,
            Event::DocumentStart(false),
            Event::SequenceStart(0, None),
            Event::MappingStart(0, None),
            Event::Scalar("~".to_string(), TScalarStyle::Plain, 0, None),
            Event::SequenceStart(0, None),
            Event::MappingStart(0, None),
            Event::Scalar("~".to_string(), TScalarStyle::Plain, 0, None),
            Event::Scalar("~".to_string(), TScalarStyle::Plain, 0, None),
            Event::MappingEnd,
            Event::SequenceEnd,
            Event::MappingEnd,
            Event::SequenceEnd,
            Event::DocumentEnd,
            Event::StreamEnd,
        ]
    );

    // Interleaved nested implicit and non-implicit mappings.
    assert_eq!(
        run_parser("[a: [  [b:]]]").unwrap(),
        //          ^   ^  ^ has an implicit mapping
        //          |   ` has no implicit mapping
        //          ` has an implicit mapping
        // We must make sure that the `MappingEnd` events are correctly issued for the first and
        // third nested sequences, but not the second.
        [
            Event::StreamStart,
            Event::DocumentStart(false),
            Event::SequenceStart(0, None),
            Event::MappingStart(0, None),
            Event::Scalar("a".to_string(), TScalarStyle::Plain, 0, None),
            Event::SequenceStart(0, None),
            // No `MappingStart` here.
            Event::SequenceStart(0, None),
            Event::MappingStart(0, None),
            Event::Scalar("b".to_string(), TScalarStyle::Plain, 0, None),
            Event::Scalar("~".to_string(), TScalarStyle::Plain, 0, None),
            Event::MappingEnd,
            Event::SequenceEnd,
            // No `MappingEnd` here.
            Event::SequenceEnd,
            Event::MappingEnd,
            Event::SequenceEnd,
            Event::DocumentEnd,
            Event::StreamEnd,
        ]
    );

    // There needs to be a space between a `:` in a flow sequence and the value.
    assert!(run_parser("[:[:]]").is_err());
    assert!(run_parser("[a:[42]]").is_err());

    // Double-quoted keys may have a value without a space for JSON-compatibility.
    assert_eq!(
        run_parser(r#"["a":[]]"#).unwrap(),
        [
            Event::StreamStart,
            Event::DocumentStart(false),
            Event::SequenceStart(0, None),
            Event::MappingStart(0, None),
            Event::Scalar("a".to_string(), TScalarStyle::DoubleQuoted, 0, None),
            Event::SequenceStart(0, None),
            Event::SequenceEnd,
            Event::MappingEnd,
            Event::SequenceEnd,
            Event::DocumentEnd,
            Event::StreamEnd,
        ]
    );
}

#[test]
fn test_issue13() {
    let s = "{---";
    let Err(error) = run_parser(s) else { panic!() };
    assert_eq!(
        error.info(),
        "while parsing a flow mapping, did not find expected ',' or '}'"
    );
    assert_eq!(
        error.to_string(),
        "while parsing a flow mapping, did not find expected ',' or '}' at byte 1 line 1 column 2"
    );
}
