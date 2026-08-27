[Home](../../) / [Examples](sort-driver.md) / Card Catalog

# Card Catalog

Source: [examples/card_catalog](https://github.com/johnjoeallen/bascal/tree/main/examples/card_catalog)

The [Card Catalog](https://github.com/johnjoeallen/bascal/tree/main/examples/card_catalog) is the flagship [record/file DSL](../manual/record-files.md) application. `Header` and `Entry` share one random-access file; procedures add, list, search, and delete entries from procedure scope. It is adapted from Carlos A. Lujan S.'s 1983 `CLERK.BAS`.

```text
bcc examples/card_catalog/card_catalog.bcl
fbc -lang qb examples/card_catalog/card_catalog.bas -x tmp/card_catalog
./tmp/card_catalog
```

The program is interactive; follow the on-screen menu.

[← REMLINE](remline.md) [Next: Adventure Game →](adventure.md)
