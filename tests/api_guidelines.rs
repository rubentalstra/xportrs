//! Rust API Guidelines compliance tests.
//!
//! These tests verify that xportrs follows the Rust API Guidelines.
//! See: <https://rust-lang.github.io/api-guidelines/checklist.html>

use xportrs::{
    Agency, Column, ColumnData, ColumnNames, Dataset, DomainCode, Error, IntoIter, Issue, Iter,
    IterMut, Label, Severity, TextMode, ValidatedWrite, VariableName, VariableRole, Verbosity,
    XptReaderBuilder, XptVersion, XptWriterBuilder,
};

// =============================================================================
// C-SEND-SYNC: Types are Send and Sync where possible
// =============================================================================

/// Compile-time test that a type is Send.
fn assert_send<T: Send>() {}

/// Compile-time test that a type is Sync.
fn assert_sync<T: Sync>() {}

#[test]
fn public_types_are_send() {
    // Data types
    assert_send::<Dataset>();
    assert_send::<Column>();
    assert_send::<ColumnData>();
    assert_send::<DomainCode>();
    assert_send::<Label>();
    assert_send::<VariableName>();

    // Enums
    assert_send::<VariableRole>();
    assert_send::<Agency>();
    assert_send::<XptVersion>();
    assert_send::<Severity>();
    assert_send::<Issue>();
    assert_send::<TextMode>();
    assert_send::<Verbosity>();

    // Error type
    assert_send::<Error>();
}

#[test]
fn public_types_are_sync() {
    // Data types
    assert_sync::<Dataset>();
    assert_sync::<Column>();
    assert_sync::<ColumnData>();
    assert_sync::<DomainCode>();
    assert_sync::<Label>();
    assert_sync::<VariableName>();

    // Enums
    assert_sync::<VariableRole>();
    assert_sync::<Agency>();
    assert_sync::<XptVersion>();
    assert_sync::<Severity>();
    assert_sync::<Issue>();
    assert_sync::<TextMode>();
    assert_sync::<Verbosity>();

    // Error type
    assert_sync::<Error>();
}

// =============================================================================
// C-DEBUG: All public types implement Debug
// =============================================================================

/// Compile-time test that a type implements Debug.
fn assert_debug<T: std::fmt::Debug>() {}

#[test]
fn public_types_implement_debug() {
    // Data types
    assert_debug::<Dataset>();
    assert_debug::<Column>();
    assert_debug::<ColumnData>();
    assert_debug::<DomainCode>();
    assert_debug::<Label>();
    assert_debug::<VariableName>();

    // Enums
    assert_debug::<VariableRole>();
    assert_debug::<Agency>();
    assert_debug::<XptVersion>();
    assert_debug::<Severity>();
    assert_debug::<Issue>();
    assert_debug::<TextMode>();
    assert_debug::<Verbosity>();

    // Error type
    assert_debug::<Error>();

    // Builders and result types
    assert_debug::<XptReaderBuilder>();
    assert_debug::<XptWriterBuilder>();
    assert_debug::<ValidatedWrite>();
}

// =============================================================================
// C-DEBUG-NONEMPTY: Debug representation is never empty
// =============================================================================

#[test]
fn debug_representation_is_never_empty() {
    // Test that Debug output is non-empty for various types
    let domain_code = DomainCode::new("AE");
    let debug_str = format!("{:?}", domain_code);
    assert!(
        !debug_str.is_empty(),
        "DomainCode Debug should not be empty"
    );

    let label = Label::new("Test Label");
    let debug_str = format!("{:?}", label);
    assert!(!debug_str.is_empty(), "Label Debug should not be empty");

    let variable_name = VariableName::new("USUBJID");
    let debug_str = format!("{:?}", variable_name);
    assert!(
        !debug_str.is_empty(),
        "VariableName Debug should not be empty"
    );

    let severity = Severity::Error;
    let debug_str = format!("{:?}", severity);
    assert!(!debug_str.is_empty(), "Severity Debug should not be empty");

    let agency = Agency::FDA;
    let debug_str = format!("{:?}", agency);
    assert!(!debug_str.is_empty(), "Agency Debug should not be empty");

    let version = XptVersion::V5;
    let debug_str = format!("{:?}", version);
    assert!(
        !debug_str.is_empty(),
        "XptVersion Debug should not be empty"
    );

    // Empty collections should still have non-empty Debug
    let empty_data = ColumnData::F64(vec![]);
    let debug_str = format!("{:?}", empty_data);
    assert!(
        !debug_str.is_empty(),
        "Empty ColumnData Debug should not be empty"
    );

    // Empty dataset
    let empty_dataset = Dataset::new("AE", vec![]).unwrap();
    let debug_str = format!("{:?}", empty_dataset);
    assert!(
        !debug_str.is_empty(),
        "Empty Dataset Debug should not be empty"
    );
}

// =============================================================================
// C-GOOD-ERR: Error types are meaningful and well-behaved
// =============================================================================

/// Compile-time test that Error implements `std::error::Error`.
fn assert_error<T: std::error::Error>() {}

#[test]
fn error_implements_std_error() {
    assert_error::<Error>();
}

#[test]
fn error_is_send_sync_static() {
    fn assert_error_bounds<T: std::error::Error + Send + Sync + 'static>() {}
    assert_error_bounds::<Error>();
}

// =============================================================================
// C-COMMON-TRAITS: Types eagerly implement common traits
// =============================================================================

/// Compile-time test for Clone.
fn assert_clone<T: Clone>() {}

/// Compile-time test for `PartialEq`.
fn assert_partial_eq<T: PartialEq>() {}

/// Compile-time test for Eq.
fn assert_eq<T: Eq>() {}

/// Compile-time test for Hash.
fn assert_hash<T: std::hash::Hash>() {}

/// Compile-time test for Display.
fn assert_display<T: std::fmt::Display>() {}

/// Compile-time test for Default.
fn assert_default<T: Default>() {}

#[test]
fn common_traits_clone() {
    assert_clone::<Dataset>();
    assert_clone::<Column>();
    assert_clone::<ColumnData>();
    assert_clone::<DomainCode>();
    assert_clone::<Label>();
    assert_clone::<VariableName>();
    assert_clone::<VariableRole>();
    assert_clone::<Agency>();
    assert_clone::<XptVersion>();
    assert_clone::<Severity>();
    assert_clone::<Issue>();
}

#[test]
fn common_traits_partial_eq() {
    assert_partial_eq::<Dataset>();
    assert_partial_eq::<Column>();
    assert_partial_eq::<ColumnData>();
    assert_partial_eq::<DomainCode>();
    assert_partial_eq::<Label>();
    assert_partial_eq::<VariableName>();
    assert_partial_eq::<VariableRole>();
    assert_partial_eq::<Agency>();
    assert_partial_eq::<XptVersion>();
    assert_partial_eq::<Severity>();
    assert_partial_eq::<Issue>();
}

#[test]
fn common_traits_eq() {
    // Types that should implement Eq (no f64)
    assert_eq::<DomainCode>();
    assert_eq::<Label>();
    assert_eq::<VariableName>();
    assert_eq::<VariableRole>();
    assert_eq::<Agency>();
    assert_eq::<XptVersion>();
    assert_eq::<Severity>();
    assert_eq::<Issue>();

    // Note: Dataset, Column, ColumnData do NOT implement Eq because they contain f64
}

#[test]
fn common_traits_hash() {
    // Types that should implement Hash
    assert_hash::<DomainCode>();
    assert_hash::<Label>();
    assert_hash::<VariableName>();
    assert_hash::<VariableRole>();
    assert_hash::<Agency>();
    assert_hash::<XptVersion>();
    assert_hash::<Severity>();

    // Note: Dataset, Column, ColumnData, Issue do NOT implement Hash
}

#[test]
fn common_traits_display() {
    assert_display::<Dataset>();
    assert_display::<Column>();
    assert_display::<ColumnData>();
    assert_display::<DomainCode>();
    assert_display::<Label>();
    assert_display::<VariableName>();
    assert_display::<VariableRole>();
    assert_display::<Agency>();
    assert_display::<XptVersion>();
    assert_display::<Severity>();
    assert_display::<Issue>();
    assert_display::<Error>();
}

#[test]
fn display_format_dataset() {
    let dataset = Dataset::new(
        "AE",
        vec![
            Column::new("A", ColumnData::F64(vec![Some(1.0), Some(2.0)])),
            Column::new(
                "B",
                ColumnData::String(vec![Some("x".into()), Some("y".into())]),
            ),
        ],
    )
    .unwrap();

    let display = format!("{}", dataset);
    assert!(display.contains("AE"), "Display should contain domain code");
    assert!(
        display.contains("2 rows"),
        "Display should contain row count"
    );
    assert!(
        display.contains("2 cols"),
        "Display should contain column count"
    );
}

#[test]
fn display_format_column() {
    let col = Column::new("USUBJID", ColumnData::String(vec![Some("01-001".into())]));
    let display = format!("{}", col);
    assert!(
        display.contains("USUBJID"),
        "Display should contain column name"
    );
    assert!(
        display.contains("String"),
        "Display should contain data type"
    );
}

#[test]
fn display_format_column_data() {
    let data = ColumnData::F64(vec![Some(1.0), Some(2.0), Some(3.0)]);
    let display = format!("{}", data);
    assert!(display.contains("F64"), "Display should contain type");
    assert!(display.contains("3"), "Display should contain length");
}

#[test]
fn common_traits_default() {
    assert_default::<XptVersion>();
    assert_default::<TextMode>();
    assert_default::<Verbosity>();
}

// =============================================================================
// C-COMMON-TRAITS: Ord/PartialOrd where meaningful
// =============================================================================

/// Compile-time test for Ord.
fn assert_ord<T: Ord>() {}

#[test]
fn common_traits_ord() {
    // Types that have natural ordering
    assert_ord::<Severity>(); // Info < Warning < Error
    assert_ord::<XptVersion>(); // V5 < V8

    // Note: Agency, VariableRole do NOT have Ord (no natural ordering)
}

#[test]
fn severity_ordering() {
    assert!(Severity::Info < Severity::Warning);
    assert!(Severity::Warning < Severity::Error);
    assert!(Severity::Info < Severity::Error);
}

#[test]
fn xpt_version_ordering() {
    assert!(XptVersion::V5 < XptVersion::V8);
}

// =============================================================================
// C-CONV: Ad-hoc conversions follow as_, to_, into_ conventions
// =============================================================================

#[test]
fn conversion_naming_conventions() {
    let domain_code = DomainCode::new("AE");

    // as_ = free, borrowed -> borrowed
    let _: &str = domain_code.as_str();

    // into_ = consumes self, ownership transfer
    let _: String = domain_code.into_inner();

    let label = Label::new("Test");
    let _: &str = label.as_str();
    let _: String = label.into_inner();

    let variable_name = VariableName::new("AGE");
    let _: &str = variable_name.as_str();
    let _: String = variable_name.into_inner();
}

// =============================================================================
// C-GETTER: Getter names follow Rust convention (no get_ prefix)
// =============================================================================

#[test]
fn getter_naming_conventions() {
    let dataset = Dataset::new(
        "AE",
        vec![Column::new(
            "USUBJID",
            ColumnData::String(vec![Some("01-001".into())]),
        )],
    )
    .unwrap();

    // Getters use field name directly, not get_field_name
    let _: &str = dataset.domain_code();
    let _: Option<&str> = dataset.dataset_label();
    let _: &[Column] = dataset.columns();
    let _: usize = dataset.nrows();
    let _: usize = dataset.ncols();

    let column = &dataset.columns()[0];
    let _: &str = column.name();
    let _: Option<VariableRole> = column.role();
    let _: &ColumnData = column.data();
}

// =============================================================================
// C-CTOR: Constructors are static, inherent methods
// =============================================================================

#[test]
fn constructor_conventions() {
    // Primary constructor is new()
    let _ = DomainCode::new("AE");
    let _ = Label::new("Test");
    let _ = Column::new("VAR", ColumnData::F64(vec![]));

    // Secondary constructors use _with_ suffix
    let _ = Column::with_role("VAR", VariableRole::Identifier, ColumnData::F64(vec![]));
    let _ = Dataset::with_label("AE", Some("Label"), vec![]);
}

// =============================================================================
// C-ITER-TY: Iterator type names match the methods that produce them
// =============================================================================

#[test]
fn iterator_type_names() {
    let dataset = Dataset::new(
        "AE",
        vec![Column::new(
            "USUBJID",
            ColumnData::String(vec![Some("01-001".into())]),
        )],
    )
    .unwrap();

    // iter() returns Iter
    let _: Iter<'_> = dataset.iter();

    // column_names() returns ColumnNames
    let _: ColumnNames<'_> = dataset.column_names();

    // into_iter() returns IntoIter
    let _: IntoIter = dataset.clone().into_iter();
}

#[test]
fn iterator_types_are_debug() {
    assert_debug::<Iter<'_>>();
    assert_debug::<IterMut<'_>>();
    assert_debug::<IntoIter>();
    assert_debug::<ColumnNames<'_>>();
}

#[test]
fn iterator_types_are_send_sync() {
    assert_send::<Iter<'_>>();
    assert_sync::<Iter<'_>>();
    assert_send::<IterMut<'_>>();
    assert_sync::<IterMut<'_>>();
    assert_send::<IntoIter>();
    assert_sync::<IntoIter>();
    assert_send::<ColumnNames<'_>>();
    assert_sync::<ColumnNames<'_>>();
}

// =============================================================================
// C-COLLECT: Collections implement FromIterator and Extend
// =============================================================================

#[test]
fn dataset_implements_index_by_position() {
    let dataset = Dataset::new(
        "AE",
        vec![
            Column::new("A", ColumnData::F64(vec![Some(1.0)])),
            Column::new("B", ColumnData::F64(vec![Some(2.0)])),
        ],
    )
    .unwrap();

    assert_eq!(dataset[0].name(), "A");
    assert_eq!(dataset[1].name(), "B");
}

#[test]
fn dataset_implements_index_by_name() {
    let dataset = Dataset::new(
        "AE",
        vec![
            Column::new("USUBJID", ColumnData::String(vec![Some("01-001".into())])),
            Column::new("AESEQ", ColumnData::F64(vec![Some(1.0)])),
        ],
    )
    .unwrap();

    assert_eq!(dataset["USUBJID"].name(), "USUBJID");
    assert_eq!(dataset["AESEQ"].name(), "AESEQ");
}

#[test]
fn dataset_implements_extend() {
    let mut dataset = Dataset::new(
        "AE",
        vec![Column::new("A", ColumnData::F64(vec![Some(1.0)]))],
    )
    .unwrap();

    dataset.extend([
        Column::new("B", ColumnData::F64(vec![Some(2.0)])),
        Column::new("C", ColumnData::F64(vec![Some(3.0)])),
    ]);

    assert_eq!(dataset.ncols(), 3);
    assert_eq!(dataset[1].name(), "B");
    assert_eq!(dataset[2].name(), "C");
}

// =============================================================================
// C-VALIDATE: Functions validate their arguments
// =============================================================================

#[test]
fn validation_at_construction() {
    // Dataset validates column lengths match
    let result = Dataset::new(
        "AE",
        vec![
            Column::new("A", ColumnData::F64(vec![Some(1.0)])),
            Column::new("B", ColumnData::F64(vec![Some(1.0), Some(2.0)])),
        ],
    );
    assert!(result.is_err(), "Should reject mismatched column lengths");
}
