# DuckDB JFR extension

This is a custom extension for [DuckDB](https://duckdb.org/) to directly read Java Flight Recorder (JFR) files.

## Development

### CLion

- Install [Bear](https://github.com/rizsotto/Bear)
- Generate `compile_commands.json` by `bear -- cargo build`
  * build.rs has to re-run for generation
  * You may need to clean the project or make some change to cpp sources if necessary
- Open the project in CLion
