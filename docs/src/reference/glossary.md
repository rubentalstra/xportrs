# Glossary

This glossary defines key terms used in xportrs and clinical trial data management.

## A

ADaM (Analysis Data Model)
: CDISC standard for analysis-ready datasets derived from SDTM data. Common datasets include ADSL (subject-level), ADAE (adverse events analysis), and ADLB (laboratory analysis).

Agency
: Regulatory authority that reviews drug submissions. Major agencies include FDA (US), PMDA (Japan), NMPA (China), and EMA (Europe).

ANDA (Abbreviated New Drug Application)
: FDA submission type for generic drugs.

ASCII
: American Standard Code for Information Interchange. Character encoding required by FDA for XPT file text content. Uses bytes 0x00-0x7F.

## B

BLA (Biologics License Application)
: FDA submission type for biological products.

Big-endian
: Byte order where the most significant byte is stored first. Used in XPT files.

## C

CDASH (Clinical Data Acquisition Standards Harmonization)
: CDISC standard for data collection forms. Upstream of SDTM.

CDISC (Clinical Data Interchange Standards Consortium)
: Organization that develops data standards for clinical research, including SDTM, ADaM, and controlled terminology.

Column
: In xportrs, represents a variable with its data and metadata. Corresponds to a variable in XPT terminology.

ColumnData
: Enum in xportrs representing typed data storage (F64, String, Date, etc.).

Controlled Terminology
: CDISC-defined standard values for coded variables. Example: SEX must be M, F, U, or UNDIFFERENTIATED.

## D

Dataset
: In xportrs, a collection of columns representing an XPT member. Also called a domain in SDTM context.

Define-XML
: XML file describing the metadata for CDISC datasets. Required alongside XPT files in submissions.

Domain
: SDTM term for a dataset representing a specific type of data (DM=Demographics, AE=Adverse Events, etc.).

DomainCode
: In xportrs, the 1-8 character dataset identifier (e.g., "AE", "DM").

## E

eCTD (Electronic Common Technical Document)
: Standard format for regulatory submissions. XPT files are placed in specific eCTD modules.

EMA (European Medicines Agency)
: Regulatory authority for the European Union.

Epoch
: Reference date for date calculations. SAS uses January 1, 1960. Unix uses January 1, 1970.

## F

FDA (Food and Drug Administration)
: U.S. regulatory authority for drugs and medical devices.

Format
: In xportrs, represents a SAS display format (e.g., DATE9., 8.2, $CHAR200.).

## I

IBM Floating-Point
: Hexadecimal (base-16) floating-point format used in XPT files. Different from IEEE 754.

IND (Investigational New Drug)
: FDA application to begin clinical trials.

Informat
: SAS input format specifying how data is read. Stored in XPT NAMESTR records.

Issue
: In xportrs, represents a validation problem (Error, Warning, or Info severity).

## L

Label
: Descriptive text for a dataset or variable. Limited to 40 bytes in XPT V5.

Latin-1 (ISO-8859-1)
: Character encoding supporting Western European characters. Allowed for non-FDA submissions.

## M

Member
: XPT term for a dataset within a transport file. An XPT file can contain multiple members.

Missing Value
: XPT uses special byte patterns for missing data. Standard missing is 0x2E (period). Special missing values .A-.Z and ._ are also supported.

## N

NAMESTR
: 140-byte record in XPT files describing a variable's metadata (name, label, format, type, length).

NDA (New Drug Application)
: FDA submission type for new drugs.

NMPA (National Medical Products Administration)
: Regulatory authority for China.

## P

Pinnacle 21
: Industry-standard validation tool for CDISC compliance. Checks XPT files and define.xml.

PMDA (Pharmaceuticals and Medical Devices Agency)
: Regulatory authority for Japan.

## S

SAS
: Statistical Analysis System. Software that created the XPT format.

SAS Epoch
: January 1, 1960. Reference date for SAS date values.

SDTM (Study Data Tabulation Model)
: CDISC standard for tabulation data structure. Defines domains like DM, AE, LB, VS.

SEND (Standard for Exchange of Nonclinical Data)
: CDISC standard for nonclinical (animal) study data.

Severity
: Validation issue classification in xportrs: Error (blocks write), Warning (review recommended), Info (suggestion).

## T

TCG (Technical Conformance Guide)
: FDA document specifying electronic submission requirements.

TS-140
: SAS Technical Note defining the XPT V5 format specification.

## U

USUBJID (Unique Subject Identifier)
: Standard SDTM variable uniquely identifying a subject across all datasets.

## V

ValidatedWrite
: In xportrs, a validated dataset ready to be written to a file.

VariableName
: In xportrs, the 1-8 character variable identifier (e.g., "USUBJID").

VariableRole
: CDISC classification of variables: Identifier, Topic, Timing, Qualifier, Rule, Synonym, Record.

VariableSpec
: Internal xportrs structure containing computed write specification for a variable.

## X

XPT
: SAS Transport file format. XPT V5 is required for regulatory submissions.

XPT V5
: Version 5 of SAS Transport format (also called Version 5/6). Uses 8-byte variable names, IBM floating-point, and 80-byte records.

XPT V8
: Newer SAS Transport format with longer names and IEEE floating-point. Not accepted for FDA submissions.
