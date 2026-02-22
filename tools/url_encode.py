#!/usr/bin/env python3
"""
URL percent-encode a string for use in jankenoboe CLI --term and --data values.

Usage:
    python3 tools/url_encode.py "it's a test"
    python3 tools/url_encode.py 'Fuwa Fuwa Time (5-nin Ver.)'
    python3 tools/url_encode.py "The \"Best\" Artist"

Output:
    it%27s%20a%20test
    Fuwa%20Fuwa%20Time%20%285-nin%20Ver.%29
    The%20%22Best%22%20Artist

The encoded output can be used directly in JSON values for
--term and --data:
    jankenoboe search artist --fields id,name \
        --term '{"name":{"value":"it%27s%20a%20test"}}'
    jankenoboe create artist \
        --data '{"name":"Ado%27s%20Music"}'
"""

import sys
from urllib.parse import quote


def url_encode(text: str) -> str:
    """Encode text using URL percent-encoding (safe: unreserved chars only)."""
    return quote(text, safe="")


def main():
    if len(sys.argv) < 2:
        print("Usage: python3 tools/url_encode.py <text>", file=sys.stderr)
        print(
            'Example: python3 tools/url_encode.py "it\'s a test"',
            file=sys.stderr,
        )
        sys.exit(1)

    text = sys.argv[1]
    print(url_encode(text))


if __name__ == "__main__":
    main()
