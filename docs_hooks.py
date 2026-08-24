"""MkDocs hook registering BASCAL with Pygments."""

from pathlib import Path
import sys


def on_config(config, **_kwargs):
    docs_dir = Path(config.docs_dir).resolve()
    sys.path.insert(0, str(docs_dir))
    from bascal_lexer import BascalLexer
    from pygments.lexers import _lexer_cache
    from pygments.lexers._mapping import LEXERS

    _lexer_cache["BascalLexer"] = BascalLexer
    LEXERS["BascalLexer"] = ("bascal_lexer", "BASCAL", ("bascal", "bcl"), ("*.bcl",), ())
    return config


def on_post_build(config, **_kwargs):
    """Keep the former single-page manual URL alive after the migration."""
    (Path(config.site_dir) / "manual.html").write_text(
        '<!doctype html><meta http-equiv="refresh" content="0; url=manual/">'
        '<a href="manual/">The BASCAL manual has moved.</a>\n'
    )
