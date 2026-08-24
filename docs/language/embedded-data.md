## Shipping small constant tables with `data`

Classic BASIC’s `data`/`read` pair embeds a small constant table directly in the program instead of a separate file. It suits fixed lookup tables for which a record file would be overkill.

```bascal
dim country$(2), capital$(2)
declare i%

for i% = 0 to 2
    read country$(i%), capital$(i%)
end for

print country$(0); " "; capital$(0)

data "USA", "Washington"
data "France", "Paris"
data "Japan", "Tokyo"
```

`data` statements can appear anywhere in the source. The compiler gathers them into one sequential table regardless of where they are written. Each `read` takes the next value, left to right, matching each variable’s declared type.

## Rewinding with `restore`

`restore` rewinds to the first `data` statement. `restore label` rewinds to the `data` statement immediately following that label, letting one program contain more than one named table.

```bascal
countries:
data "USA", "France", "Japan"

restore countries
read first$
print first$
```

[← Errors and labels](errors-labels-and-data.md)[The standard library →](standard-library.md)
