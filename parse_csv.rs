#[derive(PartialEq)]
enum CsvFieldState {
    Unquoted,
    Quoted,
    AfterQuote,
}
