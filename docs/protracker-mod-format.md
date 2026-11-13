# ProTracker MOD-formatet byte för byte (31 samplingar, 4 kanaler)

Den här referensen beskriver standardformatet för 4-kanaliga ProTracker 2.3D `.mod`-filer (`M.K.`), så att en läsare och skrivare kan hantera befintliga filer förlustfritt (round-trip) och generera filer som fungerar i `pt2-clone` och på en riktig Commodore Amiga (Paula-chipet).

Alla 16-bitars heltal i headern är **big-endian** (Motorola 68000-byteordning).

---

## 1. Övergripande fillayout

Layout (endast översikt, inte ett kommando):
```text
+-------------------------------------------------------------------+
| 0 .. 1083 (1084 byte)     Header (låtnamn, 31 samplingar, order)  |
+-------------------------------------------------------------------+
| 1084 .. +N*1024           Mönsterdata (N mönster × 1024 byte)     |
+-------------------------------------------------------------------+
| Efter mönsterdata .. EOF  Samplingsdata (8-bit signed PCM)        |
+-------------------------------------------------------------------+
```

### Hur räknar man ut antal mönster `N`?
Läs hela mönstertabellen på byte `952..1079` (alla 128 positioner, inte bara upp till låtlängden):
- `N = max(order[0..127]) + 1`
- Varje mönster är `64 rader × 4 kanaler × 4 byte = 1024 byte`.

---

## 2. Filheader (1084 byte, offset `0..1083`)

| Offset (dec) | Längd (byte) | Fält | Beskrivning |
| :--- | :--- | :--- | :--- |
| `0..19` | `20` | Song Title | ASCII-sträng, utfylld med `0x00`. |
| `20..949` | `930` (`31 × 30`) | Sample Headers `1..31` | 30 byte per sampling (se tabell nedan). |
| `950` | `1` | Song Length | Antal positioner i låten (`1..128`). |
| `951` | `1` | Restart / Historic Byte | Sätts till `127` (`0x7F`) i ProTracker. |
| `952..1079` | `128` | Pattern Order Table | Vilket mönster (`0..63`) som spelas på position `0..127`. |
| `1080..1083` | `4` | Format Tag | `M.K.` (standard <= 64 mönster), `M!K!` (> 64 mönster), `FLT4` eller `4CHN`. |

### Samplingsheader (30 byte per sampling, upprepas 31 gånger på offset `20..949`)

För sampling `s` (`1..31`) börjar posten vid `offset = 20 + (s - 1) * 30`:

| Relativ offset | Längd (byte) | Typ | Fält | Beskrivning |
| :--- | :--- | :--- | :--- | :--- |
| `+0..21` | `22` | ASCII | Sample Name | Namn/kommentar, utfyllt med `0x00`. |
| `+22..23` | `2` | `uint16 BE` | Sample Length | Längd i **16-bitars ord** (`byte_length = words * 2`). `0` = tom sampling. |
| `+24` | `1` | `uint8` | Finetune | Lägre 4 bitarna (`0x0..0xF`). Tolkat som 4-bitars tvåkomplement: `0..7` = `0..+7`, `8..15` = `-8..-1` (i åttondels halvtoner). Övre 4 bitarna ska vara `0`. |
| `+25` | `1` | `uint8` | Volume | Standardvolym `0..64` (`0x00..0x40`). |
| `+26..27` | `2` | `uint16 BE` | Repeat Offset | Loop-start i **16-bitars ord** (`byte_offset = words * 2`). |
| `+28..29` | `2` | `uint16 BE` | Repeat Length | Loop-längd i **16-bitars ord** (`byte_length = words * 2`). Värde `0` eller `1` betyder **ingen loop** (ProTracker skriver `0x0001`). Värde `> 1` betyder aktiv loop. |

---

## 3. Mönsterdata (1024 byte per mönster)

Startar på byte `1084`. Varje mönster består av:
- **64 rader** (`row 0..63`)
- **4 kanaler per rad** (`ch 0..3`, motsvarar Amiga Paula-kanalerna 1–4: Vänster, Höger, Höger, Vänster — LRRL)
- **4 byte per notcell** (`b0, b1, b2, b3`)

### Uppackning av en 4-bytes notcell (`b0, b1, b2, b3`)

Bitlayout för en notcell (endast illustration, inte ett kommando):
```text
Byte 0 (b0)       Byte 1 (b1)       Byte 2 (b2)       Byte 3 (b3)
[s4 s3 s2 s1 | p11..p8] [   p7 .. p0   ] [s0 c3 c2 c1 | e3..e0] [   x3..x0 y3..y0  ]
 \_________/   \______/  \____________/   \_________/   \____/   \________________/
  Sample hi    Period hi    Period lo      Sample lo    Effekt     Effekt-parameter
```

- **Samplingsnummer (`0..31`):**
  `sample = (b0 & 0xF0) | ((b2 >> 4) & 0x0F)`
  (`0` betyder att föregående samplingsnummer på kanalen behålls.)
- **Period (`0..4095`, 12 bitar):**
  `period = ((b0 & 0x0F) << 8) | b1`
  (`0` betyder att ingen ny ton triggas på den här raden.)
- **Effektkommando (`0x0..0xF`, 4 bitar):**
  `effect_cmd = b2 & 0x0F`
- **Effektparameter (`0x00..0xFF`, 8 bitar):**
  `effect_arg = b3` (`x = (b3 >> 4) & 0x0F`, `y = b3 & 0x0F`)

### Packning tillbaka till 4 byte
- `b0 = (sample & 0xF0) | ((period >> 8) & 0x0F)`
- `b1 = period & 0xFF`
- `b2 = ((sample & 0x0F) << 4) | (effect_cmd & 0x0F)`
- `b3 = effect_arg & 0xFF`

---

## 4. Amiga Period-tabell (Finetune 0)

ProTracker lagrar inte MIDI-notnummer utan Amiga Paula-periodvärden. Giltigt omfång i ProTracker är 3 oktaver (`C-1` till `B-3`, 36 toner):

| Oktav | C | C# | D | D# | E | F | F# | G | G# | A | A# | B |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **Oktav 1** | 856 | 808 | 762 | 720 | 678 | 640 | 604 | 570 | 538 | 508 | 480 | 453 |
| **Oktav 2** | 428 | 404 | 381 | 360 | 339 | 320 | 302 | 285 | 269 | 254 | 240 | 226 |
| **Oktav 3** | 214 | 202 | 190 | 180 | 170 | 160 | 151 | 143 | 135 | 127 | 120 | 113 |

---

## 5. Vanliga ProTracker-effekter (`0xy`–`Fxx`)

| Kod | Namn | Betydelse |
| :--- | :--- | :--- |
| `0xy` | Arpeggio | Växlar snabbt mellan grundton, `+x` halvtoner och `+y` halvtoner (t.ex. `037` = mollackord, `047` = durackord). Om `xy = 00`: ingen effekt. |
| `1xx` | Portamento Up | Glider tonhöjden uppåt med hastighet `xx`. |
| `2xx` | Portamento Down | Glider tonhöjden nedåt med hastighet `xx`. |
| `3xx` | Tone Portamento | Glider mot målnote med hastighet `xx`. |
| `4xy` | Vibrato | Vibrato med hastighet `x` och djup `y`. |
| `9xx` | Sample Offset | Startar uppspelning av samplingen vid byteoffset `xx * 256`. |
| `Axy` | Volume Slide | Höjer volymen med `x` eller sänker med `y` per tick. |
| `Bxx` | Position Jump | Hoppar till position `xx` i Pattern Order Table. |
| `Cxx` | Set Volume | Sätter kanalens volym direkt till `xx` (`00..40` hex = `0..64` dec). |
| `Dxx` | Pattern Break | Bryter nuvarande mönster och hoppar till rad `(x * 10) + y` i nästa mönster. |
| `E9x` | Retrig Note | Triggar om noten var `x`:e tick (klassiskt för trumvirvlar). |
| `ECx` | Note Cut | Klipper volymen till 0 vid tick `x`. |
| `EDx` | Note Delay | Fördröjer nottriggning med `x` ticks (används för shuffle/swing). |
| `Fxx` | Set Speed / BPM | `01..1F` (`1..31`) sätter **ticks per rad** (standard `06`). `20..FF` (`32..255`) sätter **BPM** (standard `7D` = 125 BPM). |

---

## 6. Samplingsdata (PCM)

Direkt efter det sista mönstret (vid `offset = 1084 + N * 1024`) följer rå ljuddata för sampling `1..31` i ordning:
- Format: **8-bitars signed PCM** (`int8`, `-128..+127`), råa byte utan extra headers.
- Längd i byte för sampling `s`: `sample_length_words[s] * 2`.
- **Amiga Paula-detalj:** För samplingar utan loop (`repeat_length_words <= 1`) nollställs normalt de **första 2 byten** (`0x00, 0x00`) eftersom Paula-chipet vilar på en 1-ords tyst loop i början av samplingen när den spelat klart.
