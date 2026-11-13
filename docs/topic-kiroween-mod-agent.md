# Kiroween: MOD-agenten
_Från chatt 2026-11-12_

## Var vi är
Jag är registrerad i Kiros Halloween-tävling Kiroween 2026 på Devpost. Min idé är en AI-agent som läser och skriver ProTracker MOD-filer: jag säger vad jag vill ha, agenten fyller i mönstren, och jag öppnar filen i en ProTracker-klon och ändrar själv. Låten ska gå att spela på en riktig Amiga. Min första begäran om gratis Kiro-krediter fick nej för att beskrivningen var för kort, och jag har skickat in en ny med en längre beskrivning.

## Bestämt
- Jag kör på MOD, inte SID, de här tre veckorna.
- Jag skickade in en ny kreditbegäran med kategori Resurrection och beskrivningen av MOD-agenten.
- Ljuden kan komma från egna samplingar, eller från AI om man vill.
- ProTracker 2-klonen (`pt2-clone-macos.app` från https://16-bits.org/) är installerad på Macen och testad med en nedladdad `.mod`-fil.

## Fakta vi kom fram till
- Sista dag: fredag 4 december 2026 kl. 14:00 amerikansk västkusttid, alltså 23:00 svensk tid.
- Bara nya projekt, påbörjade 30 oktober eller senare. Därför går inte HRV-appen eller IVR-agenten.
- Kategorier: Resurrection, Frankenstein, Skeleton Crew, Costume Contest.
- Domarna bedömer hur väl man använder Kiros verktyg. Mappen /.kiro får inte ligga i .gitignore.
- MOD-formatet: fyra kanaler, 8-bitars samplingar, mönster med 64 rader (se [protracker-mod-format.md](protracker-mod-format.md)).
- ProTracker 2-klon för modern dator: https://16-bits.org/ (BSD-licens med tre klausuler, Olav Sørensen) — installerad och verifierad på macOS.
- MOD-spelare med exakt Paula-ljud, bra som test: https://github.com/alfishe/amiga-paula (ingen licensfil synlig).
- Essä om trackerknepen: https://www.bosnan.net/essays/amiga-mod-music

## Fortfarande öppet
- ~~Skriva kraven i Kiro. Det är första steget.~~ → Första utkast klart i `specs/spec.md` och `specs/tickets.md`.
- ~~Hitta en beskrivning av MOD-formatet byte för byte.~~ → Dokumenterat i `docs/protracker-mod-format.md`.
- ~~Kolla om klonen finns för Mac.~~ → Installerad (`pt2-clone-macos.app`) och provspelad med `.mod`-fil.
- Väg 1 eller väg 2: agenten som eget program som läser och skriver MOD-filer, eller agenten inbyggd i ProTracker-klonen. Claude föreslog väg 1 först. Jag har inte bestämt.
- Vilket språk. Claude nämnde C++ eller Kotlin (eller Python som i detta repo). Inte bestämt.
- Läsa tävlingens regel om att bygga på någon annans kod.
- Svar på den nya kreditbegäran kommer från Devpost till min Gmail.
- En Frankenstein-röst till tävlingen (se filen om sångduetten).

## Prompt för en ny chatt
```text
Jag är med i Kiros Halloween-tävling Kiroween 2026 på Devpost. Sista dag är fredag 4 december 2026 kl. 23:00 svensk tid, och projektet måste vara nytt och byggt med Kiro. Min idé, i kategorin Resurrection: en AI-agent som läser och skriver ProTracker MOD-filer. Jag säger till exempel "lägg en bas som passar trummorna", agenten skriver noterna i mönstren, och jag öppnar filen i en ProTracker-klon (https://16-bits.org/) och ändrar själv. Låten ska gå att spela på en riktig Amiga, alltså fyra kanaler och 8-bitars samplingar. Jag har inte bestämt språk än, och inte om agenten ska vara ett eget program eller byggas in i klonen. Hjälp mig att skriva kraven i Kiro-stil som första steg, och leta upp en beskrivning av MOD-formatet byte för byte.
```
