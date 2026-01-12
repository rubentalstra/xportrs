# Data Flow

This page details how data flows through xportrs during reading and writing.

## Reading Flow

```mermaid
flowchart TB
    subgraph "1. File Parsing"
        A[XPT File] --> B[parse_header]
        B --> C[XptMemberInfo]
        C --> D[NamestrV5 records]
    end
    
    subgraph "2. Data Reading"
        D --> E[ObservationReader]
        E --> F[decode_ibm_float]
        E --> G[decode_text]
        F --> H[ObsValue::Numeric]
        G --> I[ObsValue::Character]
    end
    
    subgraph "3. Type Conversion"
        H --> J[ColumnData::F64]
        I --> K[ColumnData::String]
    end
    
    subgraph "4. Assembly"
        J --> L[Column]
        K --> L
        D --> |metadata| L
        L --> M[Dataset]
    end
```

### Step-by-Step Reading

#### 1. Parse File Header

```rust
// In parse.rs
pub fn parse_header<R: Read + Seek>(reader: &mut R) -> Result<XptInfo> {
    // Read library header (80 bytes)
    let lib_header = read_record(reader)?;
    verify_library_header(&lib_header)?;
    
    // Read each member
    let mut members = Vec::new();
    while let Some(member) = parse_member_header(reader)? {
        members.push(member);
    }
    
    Ok(XptInfo { members, ... })
}
```

#### 2. Parse NAMESTR Records

```rust
// In namestr.rs
pub fn unpack_namestr(bytes: &[u8; 140]) -> Result<NamestrV5> {
    let ntype = i16::from_be_bytes([bytes[0], bytes[1]]);
    let nlng = i16::from_be_bytes([bytes[4], bytes[5]]);
    let nname = parse_string(&bytes[8..16]);
    let nlabel = parse_string(&bytes[16..56]);
    let nform = parse_string(&bytes[56..64]);
    let nfl = i16::from_be_bytes([bytes[64], bytes[65]]);
    // ... more fields
    
    Ok(NamestrV5 { ntype, nlng, nname, nlabel, ... })
}
```

#### 3. Read Observations

```rust
// In obs.rs
pub fn read_observation(&mut self) -> Result<Option<Vec<ObsValue>>> {
    let mut row = Vec::with_capacity(self.variables.len());
    
    for var in &self.variables {
        if var.is_numeric() {
            let bytes = self.read_bytes(8)?;
            let value = decode_ibm_float(bytes);
            row.push(ObsValue::Numeric(value));
        } else {
            let bytes = self.read_bytes(var.length)?;
            let value = decode_text(bytes);
            row.push(ObsValue::Character(value));
        }
    }
    
    Ok(Some(row))
}
```

#### 4. Build Column with Metadata

```rust
// In reader.rs
let cols: Vec<Column> = member.variables.iter()
    .zip(columns)
    .map(|(var, data)| {
        let mut col = Column::new(&var.nname, data);
        
        // Transfer metadata from NAMESTR
        if !var.nlabel.is_empty() {
            col = col.with_label(var.nlabel.as_str());
        }
        if !var.nform.is_empty() {
            col = col.with_format(Format::from_namestr(
                &var.nform, var.nfl, var.nfd, var.nfj
            ));
        }
        if var.is_character() {
            col = col.with_length(var.length());
        }
        
        col
    })
    .collect();
```

## Writing Flow

```mermaid
flowchart TB
    subgraph "1. Schema Planning"
        A[Dataset] --> B[derive_schema_plan]
        B --> C[DatasetSchema]
        C --> D[VariableSpec per column]
    end
    
    subgraph "2. Validation"
        D --> E[validate_v5_schema]
        E --> F[Issue collection]
        F --> G{has_errors?}
        G --> |Yes| H[Block write]
        G --> |No| I[ValidatedWrite]
    end
    
    subgraph "3. Writing"
        I --> J[XptWriter]
        J --> K[write_headers]
        J --> L[pack_namestr]
        J --> M[write_observations]
    end
    
    subgraph "4. Encoding"
        M --> N[encode_ibm_float]
        M --> O[encode_text]
        N --> P[XPT File]
        O --> P
    end
```

### Step-by-Step Writing

#### 1. Derive Schema

```rust
// In derive.rs
pub fn derive_schema_plan(
    dataset: &Dataset,
    metadata: Option<&VariableMetadata>,
) -> DatasetSchema {
    let variables: Vec<VariableSpec> = dataset.columns()
        .iter()
        .enumerate()
        .map(|(i, col)| {
            let mut spec = VariableSpec {
                name: col.name().to_uppercase(),
                is_numeric: col.data().is_numeric(),
                length: compute_length(col),
                position: 0,  // Computed later
                ...
            };
            
            // Apply Column metadata
            if let Some(label) = col.label() {
                spec.label = label.to_string();
            }
            if let Some(format) = col.format() {
                spec.format = Some(format.clone());
            }
            
            spec
        })
        .collect();
    
    DatasetSchema { variables, ... }
}
```

#### 2. Validate

```rust
// In checks_v5.rs
pub fn validate_v5_schema(
    schema: &DatasetSchema,
    options: &WriteOptions,
) -> Vec<Issue> {
    let mut issues = Vec::new();
    
    // Dataset-level checks
    if schema.dataset_label.is_empty() {
        issues.push(Issue::MissingDatasetLabel { 
            dataset: schema.name.clone() 
        });
    }
    
    // Variable-level checks
    for var in &schema.variables {
        if var.name.len() > 8 {
            issues.push(Issue::VariableNameTooLong { ... });
        }
        if var.label.is_empty() {
            issues.push(Issue::MissingVariableLabel { 
                variable: var.name.clone() 
            });
        }
        // ... more checks
    }
    
    issues
}
```

#### 3. Pack NAMESTR

```rust
// In namestr.rs
pub fn pack_namestr<W: Write>(
    writer: &mut W,
    var: &VariableSpec,
    position: i32,
) -> Result<()> {
    // ntype
    writer.write_i16::<BigEndian>(
        if var.is_numeric { 1 } else { 2 }
    )?;
    
    // nhfun (always 0)
    writer.write_i16::<BigEndian>(0)?;
    
    // nlng
    writer.write_i16::<BigEndian>(var.length as i16)?;
    
    // nvar0
    writer.write_i16::<BigEndian>(var.index as i16 + 1)?;
    
    // nname (8 bytes, space-padded)
    let mut name = [b' '; 8];
    name[..var.name.len().min(8)]
        .copy_from_slice(var.name.as_bytes());
    writer.write_all(&name)?;
    
    // nlabel (40 bytes, space-padded)
    let mut label = [b' '; 40];
    label[..var.label.len().min(40)]
        .copy_from_slice(var.label.as_bytes());
    writer.write_all(&label)?;
    
    // Format fields
    if let Some(ref format) = var.format {
        write_format_fields(writer, format)?;
    } else {
        write_empty_format_fields(writer)?;
    }
    
    // ... remaining fields
    
    Ok(())
}
```

#### 4. Write Observations

```rust
// In writer.rs
fn write_observations<W: Write>(
    writer: &mut W,
    dataset: &Dataset,
    schema: &DatasetSchema,
) -> Result<()> {
    for row_idx in 0..dataset.nrows() {
        for (col, spec) in dataset.columns().iter()
            .zip(&schema.variables) 
        {
            if spec.is_numeric {
                let value = get_numeric_value(col, row_idx);
                let ibm = encode_ibm_float(value);
                writer.write_all(&ibm)?;
            } else {
                let value = get_string_value(col, row_idx);
                let padded = pad_to_length(&value, spec.length);
                writer.write_all(&padded)?;
            }
        }
    }
    
    // Pad to 80-byte boundary
    pad_to_record_boundary(writer)?;
    
    Ok(())
}
```

## Metadata Flow

```mermaid
sequenceDiagram
    participant User
    participant Column
    participant VariableSpec
    participant NAMESTR
    participant XPT
    
    User->>Column: with_label("Label")
    User->>Column: with_format(Format)
    
    Column->>VariableSpec: derive_schema_plan()
    Note right of VariableSpec: label, format copied
    
    VariableSpec->>NAMESTR: pack_namestr()
    Note right of NAMESTR: nlabel, nform, nfl, nfd
    
    NAMESTR->>XPT: Written to file
    
    Note over XPT,Column: Reading reverses the flow
    
    XPT->>NAMESTR: unpack_namestr()
    NAMESTR->>Column: Transfer metadata
    Note left of Column: Label, format restored
```

## Error Flow

```mermaid
flowchart TB
    A[Operation] --> B{Success?}
    B --> |Yes| C[Return Ok]
    B --> |No| D[Create Error]
    D --> E[Add context]
    E --> F[Return Err]
    F --> G{Caller handles?}
    G --> |Yes| H[Recovery/Fallback]
    G --> |No| I[Propagate up]
```

All errors are:
- Enriched with context
- `Send + Sync + 'static`
- Implement `std::error::Error`
