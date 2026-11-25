# cli-xx

`xx`: następca `x`. Właściciel określa jego polecenia jedno po drugim, korzystając z niego na co
dzień ([`cli-spec/xx.typ`](../cli-spec/xx.typ)). `x` działa obok bez zmian.

Oba programy stoją na tym samym rdzeniu (`cli-runtime`, biblioteka `xkeys-cli`, `xkeys-session`,
biblioteki wtyczek) i widzą ten sam stan: plik `~/.config/mirror/config.toml`, konteksty, skarbce,
sesje i zmienne `MIRROR_*`. `xx` nie uruchamia wtyczek `x-*`.

## Uruchomienie

```sh
cargo build                 # w tym katalogu; program trafia do wspólnego target/debug
xx --help
cargo install --path .      # albo xx w ~/.cargo/bin
```

## Polecenia

| Polecenie | Co robi |
| --- | --- |
| `xx version` | wypisuje wersję `xx` |

## Nowe polecenie

1. Właściciel opisuje polecenie; jego postać (wywołanie, flagi, pomoc) jest pokazana do akceptacji.
2. Plik `src/commands/<polecenie>.rs` (grupa: katalog), wariant w `src/command_line.rs`.
3. Logika poza warstwą poleceń leży we wspólnym rdzeniu; logika ukryta w poleceniu `x` przechodzi
   do biblioteki, z której korzystają oba programy, bez zmiany zachowania `x`.
4. Testy, wpis w tabeli wyżej i w `cli-spec/xx.typ`.

Pomoc i komunikaty po angielsku. Błąd to jedna linia `xx: …` i kod 1, błędnie wpisane polecenie
kod 2.
