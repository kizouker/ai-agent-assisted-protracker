# Tickets – AI-Assisted ProTracker (MOD-Agent)

En ticket per krav i [spec.md](spec.md), lagom stor för ett arbetspass.

---

## Ticket 1 – Binär läsare och skrivare för 4-kanaliga ProTracker `.mod`-filer
- **Requirements:** `Requirement 1` (`1.1`, `1.2`, `1.3`)
- **Priority:** Must
- **Depends on:** None
- **Description:** Bygg datamodellen och den binära parsern/serialiseraren för 31-samplingars 4-kanaliga `.mod`-filer (`M.K.`, `M!K!`, `FLT4`, `4CHN`) enligt [protracker-mod-format.md](../docs/protracker-mod-format.md).
- **Acceptance:**
  - `1.1`: Läser låttitel, 31 samplingsheaders, låtlängd, 128-positioners ordertabell, alla `N` mönster (`1024` byte styck) och all 8-bitars signed PCM-data.
  - `1.2`: Round-trip-test (`load -> save`) på en riktig `.mod`-fil i `data/` ger exakt samma byte (SHA-256 matchar).
  - `1.3`: Ogiltig format-tagg eller trunkerad fil ger ett tydligt valideringsfel med byte-offset.

---

## Ticket 2 – Textvy av låtstruktur, samplingar och 64-raders mönster
- **Requirements:** `Requirement 2` (`2.1`, `2.2`, `2.3`)
- **Priority:** Must
- **Depends on:** Ticket 1
- **Description:** Formatera en inläst `.mod`-fil som läsbar ProTracker-text (både för människa i terminalen och som kontext till AI-agenten).
- **Acceptance:**
  - `2.1`: Visar sammanfattning av låttitel, låtlängd, antal mönster, ordertabell och alla icke-tomma samplingar (`1..31`) med namn, längd, volym, finetune och loop.
  - `2.2`: Skriver ut valfritt mönster (`0..N-1`) som 64 rader × 4 kanaler i ProTracker-format (`C-2 01 C40` eller `--- 00 000`).
  - `2.3`: Periodvärden som inte finns exakt i Finetune 0-tabellen visas med närmaste notnamn + rått periodvärde.

---

## Ticket 3 – Periodtabell och Amiga/Paula-validering
- **Requirements:** `Requirement 3` (`3.1`, `3.2`, `3.3`)
- **Priority:** Must
- **Depends on:** Ticket 1
- **Description:** Konvertera mellan notnamn (`C-1`..`B-3`) och Amiga-perioder (`856`..`113`) samt validera att alla ändringar följer ProTracker 2.3D och Paula-chipets begränsningar.
- **Acceptance:**
  - `3.1`: Konverterar notsträngar (`C-1` till `B-3`) till exakta 12-bitars periodvärden enligt Finetune-tabellen.
  - `3.2`: Avvisar eller fångar ogiltiga oktaver/noter, samplingsnummer utanför `0..31`, volym utanför `0..64` och rader utanför `0..63`.
  - `3.3`: Säkerställer att de två första byten i icke-loopade samplingar är `0x00, 0x00`.

---

## Ticket 4 – AI-agent som skriver mönster utifrån naturlig språkprompt
- **Requirements:** `Requirement 4` (`4.1`, `4.2`, `4.3`)
- **Priority:** Must
- **Depends on:** Ticket 1, Ticket 2, Ticket 3
- **Description:** Koppla ihop mönsterläsaren/skrivaren med en AI-agent som tar en befintlig `.mod`-fil och en instruktion (t.ex. *"lägg en basgång på kanal 2 i mönster 0 som passar trummorna på kanal 1"*), genererar giltiga ProTracker-rader och sparar en ny `.mod`-fil.
- **Acceptance:**
  - `4.1`: Agenten får befintliga rader och samplingslista som kontext och skriver genererade noter, samplingsnummer och effekter (`0xy`–`Fxx`) till målkanalen.
  - `4.2`: Kanaler, mönster och samplingar som inte berörs av prompten lämnas helt oförändrade.
  - `4.3`: Sparar den uppdaterade `.mod`-filen (som kan öppnas och spelas direkt i `ProTracker 2.app`) och sammanfattar vilka mönster/kanaler/rader som ändrades.

---

## Ticket 5 – Skapa ny `.mod`-fil från grunden och importera 8-bitars WAV-samplingar
- **Requirements:** `Requirement 5` (`5.1`, `5.2`)
- **Priority:** Should
- **Depends on:** Ticket 1, Ticket 3
- **Description:** Gör det möjligt att skapa en ny tom `.mod`-fil och ladda in egna eller AI-genererade `.wav`-ljud som 8-bitars signed PCM-samplingar i plats `1..31`.
- **Acceptance:**
  - `5.1`: Skapar en giltig tom `M.K.`-modul med 1 tomt mönster, låtlängd `1` och standardtempo (`F06` / `F7D`).
  - `5.2`: Konverterar en WAV-fil till 8-bitars signed mono PCM och lägger in den i vald samplingsplats (`1..31`).
