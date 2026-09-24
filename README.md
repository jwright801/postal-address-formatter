# addrfmt

Postal addresses show up in forms and databases as loose blobs of text.
Half the time the state code is wrong, the ZIP is missing a digit, or
someone typed the city into the street field. addrfmt reads an address off
stdin, checks it actually looks like a deliverable address, and prints it
back in a canonical shape. It's a starting point for the kind of
validation you'd want before an address ever hits a shipping label or a
database column.

It knows the US, Canadian, and UK layouts: an optional recipient line, a
street line, an optional unit line, and a trailing line giving the city
and postal code. For US and Canada that last line is `City, Region
Postal` - the country is inferred from the region code, a US state or DC
picks the US and a Canadian province code picks Canada. UK addresses
have no region code, so that last line is instead `Post Town Postcode`
with no comma, and its shape is what picks the UK. Other countries are
not implemented yet (see Roadmap).

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

A Canadian address works the same way; the province code on the last line
is what picks the country:

```
$ printf '100 Queen St W\nToronto, ON M5H 2N2\n' | cargo run --quiet
100 Queen St W
Toronto, ON M5H 2N2

$ printf '100 Queen St W\nToronto, ON m5h2n2\n' | cargo run --quiet -- --json
{"recipient":null,"street":"100 Queen St W","unit":null,"city":"Toronto","region":"ON","postal_code":"M5H 2N2","country":"CA"}
```

A UK address has no region code, so the last line is just the post town
and postcode, in either order of spacing, with no comma:

```
$ printf '221B Baker Street\nLondon NW1 6XE\n' | cargo run --quiet
221B Baker Street
London NW1 6XE

$ printf '221B Baker Street\nLondon nw16xe\n' | cargo run --quiet -- --json
{"recipient":null,"street":"221B Baker Street","unit":null,"city":"London","region":null,"postal_code":"NW1 6XE","country":"GB"}
```

Invalid input is rejected instead of silently reformatted:

```
$ printf '500 Market St\nSan Francisco, ZZ 94105\n' | cargo run --quiet
error: 'ZZ' is not a recognized US state or Canadian province code

$ printf '500 Market St\nSan Francisco, ZZ 94105\n' | cargo run --quiet -- --json
{"error":"'ZZ' is not a recognized US state or Canadian province code"}
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

- US, Canadian, and UK addresses are parsed today. US state codes are checked
  against the full list of two-letter USPS codes (including DC); ZIP
  codes must be either 5 digits or ZIP+4. Canadian province codes are
  checked against the 13 standard abbreviations, and postal codes must
  match the `A1A 1A1` pattern (the space is optional on input, and the
  first letter is checked against the set Canada Post never issues).
- The shape of the last line is what decides which country's rules
  apply: a comma means `City, Region Postal` and picks US or CA based on
  the region code, no comma means `Post Town Postcode` and is parsed as
  UK. `City, ST ZIP`, `City, PR A1A 1A1`, and `Post Town POSTCODE` are
  all accepted from the same parser without a separate flag.
- UK postcodes are checked against the outward/inward code shape (e.g.
  `SW1A 1AA`, `EC1A 1BB`, `M1 1AE`), including the `GIR 0AA` special
  case, but not against the real list of assigned postal areas. The
  space is optional on input and letter case doesn't matter.
- Input is read as whole lines; blank lines are ignored so trailing
  newlines or extra spacing don't break parsing.
- The pretty printer rebuilds the address from the parsed fields, so it
  also normalizes whitespace and casing (region and postal codes are
  upper-cased).

## Roadmap

- multi-line street addresses (PO boxes, floor/building lines)
- a test suite covering parser edge cases
- a `--strict` flag for tighter validation (e.g. rejecting PO boxes)
- reading an address from a file argument instead of only stdin
