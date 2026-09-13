# addrfmt

Postal addresses show up in forms and databases as loose blobs of text.
Half the time the state code is wrong, the ZIP is missing a digit, or
someone typed the city into the street field. addrfmt reads an address off
stdin, checks it actually looks like a deliverable US address, and prints
it back in a canonical shape. It's a starting point for the kind of
validation you'd want before an address ever hits a shipping label or a
database column.

Right now it only knows the US layout: an optional recipient line, a
street line, an optional unit line, and a `City, ST ZIP` line. Other
countries are not implemented yet (see Roadmap).

## Usage

```
$ printf 'Jane Doe\n500 Market St\nSuite 210\nSan Francisco, CA 94105\n' | cargo run --quiet
Jane Doe
500 Market St
Suite 210
San Francisco, CA 94105
```

The recipient and unit lines are optional:

```
$ printf '500 Market St\nSan Francisco, CA 94105\n' | cargo run --quiet
500 Market St
San Francisco, CA 94105
```

Pass `--json` to get the parsed fields as a single JSON object instead of
the re-formatted text:

```
$ printf 'Jane Doe\n500 Market St\nSuite 210\nSan Francisco, CA 94105\n' | cargo run --quiet -- --json
{"recipient":"Jane Doe","street":"500 Market St","unit":"Suite 210","city":"San Francisco","region":"CA","postal_code":"94105","country":"US"}
```

Invalid input is rejected instead of silently reformatted:

```
$ printf '500 Market St\nSan Francisco, ZZ 94105\n' | cargo run --quiet
error: 'ZZ' is not a recognized two-letter state code

$ printf '500 Market St\nSan Francisco, ZZ 94105\n' | cargo run --quiet -- --json
{"error":"'ZZ' is not a recognized two-letter state code"}
```

`--json` mode always emits a JSON object on stdout, whether the address
was valid or not, which makes it safe to pipe into another program
without scraping stderr for the failure case.

## Building

Standard library only, nothing to fetch:

```
$ cargo build --release
```

## Scope

- Only US addresses are parsed today. State codes are checked against the
  full list of two-letter USPS codes (including DC). ZIP codes must be
  either 5 digits or ZIP+4.
- Input is read as whole lines; blank lines are ignored so trailing
  newlines or extra spacing don't break parsing.
- The pretty printer rebuilds the address from the parsed fields, so it
  also normalizes whitespace and casing (state codes are upper-cased).

## Roadmap

- Canadian and UK address formats
- multi-line street addresses (PO boxes, floor/building lines)
- a test suite covering parser edge cases
- a `--strict` flag for tighter validation (e.g. rejecting PO boxes)
- reading an address from a file argument instead of only stdin
