---
name: distinctive-app-design
description: >-
  Design and build web/mobile app UI that does NOT look AI-generated or "vibe-coded."
  Use this whenever building or restyling any frontend — landing pages, dashboards,
  components, forms, web apps — even when the user only says "build a page/app/component"
  and doesn't explicitly mention design, polish, or avoiding an AI look. Covers visual
  identity (fonts, color, layout, components), motion and micro-interactions,
  empty/loading/error states, and mobile viewport + keyboard handling. Default stack
  assumption: Next.js + Tailwind, but the principles are framework-agnostic.
---

# Distinctive App Design (Anti-Slop)

AI-generated UIs share a recognizable "fingerprint" because the model reaches for the
statistical average of its training data. The result looks coherent and polished but
generic — and, more damagingly, skips the functional details (states, motion, mobile
behavior) that separate a product from a prototype. This skill exists to override those
defaults with deliberate choices.

## Mindset (apply before writing any code)

- **Prompt for decisions, not aesthetics.** Don't aim for "clean and modern." Decide who
  the user is, what they need to do on each screen, and what to remove. Generic input
  produces generic output.
- **Define identity first.** Pick a personality (e.g. calm/editorial, sharp/technical,
  warm/playful) and commit. Without a constraint the output defaults to slop.
- **One strong opinion beats safe defaults.** A loud color, a bold typeface, or an
  uncommon layout — committed to fully — is more memorable than a polished page with no
  point of view. Ship something with a viewpoint, not the model's average.
- **Cosmetics get you noticed; states and mobile behavior earn trust.** Prioritize the
  functional sections below at least as much as the visual ones.

---

## 1. Visual identity — kill the defaults

These are the specific tells that out a page as AI-generated. Avoid them by default.

**Typography**
- Do NOT default to Inter. It's the "Helvetica of the LLM era" and an instant tell.
  Choose something with intent: Geist, Söhne, Untitled Sans, Haas Grotesk, or a serif
  like Tiempos / GT Sectra for headings.
- Pair a headline font with a *different* body font. Same-font everything reads as flat.
- Avoid the overused combos (Space Grotesk + Instrument Serif + Geist) and the
  "one serif-italic accent word in an otherwise-Inter hero" move.
- Avoid all-caps section labels as a reflex.

**Color**
- Avoid "VibeCode purple" — the lavender/indigo-to-purple gradient is the single most
  recognizable AI default. Purple-to-indigo gradients are the Times New Roman of AI design.
- Don't default to permanent dark mode with medium-grey body text. If dark, ensure body
  text passes WCAG AA contrast — generated dark themes routinely fail it.
- Pick a palette with a point of view: warm earth tones, high-contrast black + one bright
  accent, cream-and-pink, or restrained grey-and-blue. Then use **one dominant color, one
  accent, one neutral.** Everything else is noise.
- Skip decorative gradients, large colored glows, colored box-shadows, and aurora/radial
  "bloom" backgrounds behind heroes. They signal defaults, not design.

**Layout**
- Avoid the templated hero: centered headline in a generic sans with a badge pill floating
  right above the H1.
- Avoid colored left/top borders on cards — this stripe is one of the most reliable AI
  tells, "the em-dash of AI design."
- Avoid the grid of identical feature cards with an icon on top, numbered "1-2-3" step rows,
  stat-banner rows, and emoji-as-nav-icons.
- **Pick one layout primitive and repeat it** until it becomes the design's signature.
  This is the single highest-leverage discipline — one repeated primitive beats seven
  different card/section treatments.
- Vary spacing intentionally (tight within groups, generous between sections). Uniform
  padding everywhere feels mechanical.

**Components / icons**
- If using shadcn/ui, customize it — change the color tokens, border-radius, and shadow
  depths, and pick non-default variants. Raw shadcn defaults are a fingerprint because the
  library is built to be copy-pasted by agents.
- Avoid reflexive glassmorphism (frosted-glass cards).
- Use a consistent icon set (e.g. one line-icon library), not a mix, and never emojis as
  interface icons. Consistent iconography signals a real decision was made.

---

## 2. Motion & micro-interactions

Static UI is the "feels like a prototype" tell. But over-animation is equally amateur.

**Rules**
- **Restraint.** Three well-chosen animations beat thirty vague ones. Animate what matters;
  leave the rest static.
- **Always specify duration, easing, and trigger.** Never leave easing as linear (robotic).
  Use ease-out for entrances (fast→slow feels snappy), ease-in for exits, ease-in-out for
  toggles/loops.
- **Performance:** animate only `transform` and `opacity` (GPU-accelerated). Avoid
  animating width/height/top/left. Keep simultaneous animations under ~5 elements. Target 60fps.
- **Cap staggers.** Total stagger ≤ ~300ms regardless of list length, or a 20-item list
  takes a full second and feels broken.
- **Scroll reveals trigger once,** not every time the element re-enters. Don't slap
  `whileInView` on every element — that "popcorn" effect makes a page feel broken.

**Micro-interactions (the small feedback moments, ~200–500ms, tied to a user action)**
- Start with buttons — they're clicked constantly and a dead button feels broken. Subtle
  is key: ~2% hover scale, a brief press (scale ~0.98), snappy active state. Bigger feels
  like a toy.
- Provide feedback for every action: hover states, focus states (visible, ≥3:1 contrast),
  success checkmarks, animated toasts, live form-field validation.
- A hover-lift on cards signals clickability and removes hesitation/rage-clicks.
- Celebrate genuine milestones sparingly (onboarding complete, first item created), never
  after every step — overuse kills the signal.
- **No fake affordances:** if it looks hoverable/clickable, it must do something.

**The three that move the needle if time is short:** button hover states, loading
skeletons, and scroll reveals.

---

## 3. Off-happy-path states (empty / loading / error)

This is where AI output most reliably fails and where users lose trust. AI defaults to the
happy path (data exists, network is fast, nothing fails), yet real apps spend ~30% of time
in these states. **Design every one of them explicitly — this is not optional.**

**Empty states** — every list, feed, table, and dashboard widget needs one. A blank
container or "No data found" / "0 results" reads as a bug. A good empty state does three
things:
1. Explains *why* it's empty.
2. Gives *one* clear next action (CTA) — one, not stacked.
3. Sets expectations for what will appear once they act.
It should feel like a tour guide, not a dead end. **Fix copy by ear:** read it aloud — if
it sounds like an error ("No data"), rewrite it to sound like a friend giving directions
("Start by creating your first…"). Design distinct treatments for *no data yet* vs *error*
vs *offline* vs *search-no-results* vs *post-completion*. When it helps, dodge the empty
state entirely by preloading sample/curated content so users go empty→filled fast.

**Loading states** — use skeleton loaders (previewing the real layout) for anything over
~200ms; they make waits feel shorter. Reserve spinners for sub-200ms operations. Never show
a misleading "No records" *while still loading* and then swap in content — that destroys trust.

**Error states** — say what went wrong, why, and what to do next, in the product's own
voice. Never ship generic placeholder copy ("Something went wrong. Please try again") that
strips out reassurance exactly when the user needs it.

---

## 4. Mobile viewport & keyboard handling

The "perfect in DevTools, broken on a real phone" class of bug. A reliable tell because the
model uses the broken approach and never tests on hardware.

**Never size full-height layouts with `100vh` on mobile.** It's measured against the largest
viewport (toolbars hidden), so on load the CTA gets pushed below the fold, content hides
behind the toolbar, sections "jump" during scroll, and modals can't be reached with the
keyboard open.

**Use the modern viewport units** (Baseline since June 2025; keep a `vh` fallback for old
browsers):
- `svh` — above-the-fold content that must fit with browser chrome visible. **Default to
  this for ~90% of layouts.**
- `lvh` — immersive full-screen (splash, hero, app shell).
- `dvh` — only where you *want* real-time adaptation (e.g. a bottom-pinned input). Don't use
  `dvh` everywhere — the constant reflow feels glitchy.
- Tailwind: `min-h-svh`, `h-dvh`, etc.

**The keyboard needs a separate fix — no viewport unit reacts to it.** Add
`interactive-widget=resizes-content` to the viewport meta so the layout viewport shrinks
when the keyboard opens. In Next.js:
```ts
export const viewport = {
  maximumScale: 1,                      // also kills iOS focus auto-zoom
  interactiveWidget: 'resizes-content', // layout shrinks for the keyboard
} as const
```
For a bottom-pinned chat/composer input, `h-dvh` + `interactiveWidget: 'resizes-content'`
is the proven pattern.

**Also:**
- On focus (esp. in modals / multi-step forms where auto-scroll doesn't fire):
  `el.scrollIntoView({ behavior: 'smooth', block: 'center' })`.
- Prefer `sticky` over `fixed` for footers/action bars — `fixed` breaks more often with the
  keyboard; toggle its visibility on focus if you must use it.
- Structural reflow: `html, body { height: 100% }`, body `flex flex-col`, main `flex-1`, so
  content shifts smoothly instead of clipping.
- Safe areas (notch / Dynamic Island): `padding-bottom: env(safe-area-inset-bottom)`.
- Inputs: font-size ≥16px to avoid iOS zoom-on-focus.
- Escape hatch for hard cases: the VisualViewport / VirtualKeyboard API (keyboard height per
  frame). For most apps `dvh` + `interactive-widget` is enough — no JS observers needed.

**Testing (the step AI skips):** verify behaviors on real hardware, iOS *and* Android,
portrait *and* landscape — at load (bar expanded), after scroll (bar collapsed), and with
the keyboard open. The DevTools emulator does not simulate chrome/keyboard behavior faithfully.

---

## Pre-ship checklist

- [ ] Not Inter; headline and body fonts differ.
- [ ] No purple/indigo gradient; one dominant + one accent + one neutral; dark-mode text passes AA.
- [ ] No colored card borders, no icon-card grid clones, no emoji nav; one repeated layout primitive.
- [ ] shadcn tokens/radius/shadows customized if used.
- [ ] Motion is restrained, eased (not linear), transform/opacity only, staggers capped, reveals fire once.
- [ ] Buttons and interactive elements have hover/press/focus feedback; no fake affordances.
- [ ] Every list/table/dashboard has a designed empty state (why + one CTA + expectation).
- [ ] Skeletons for >200ms loads; error copy is specific and in-voice.
- [ ] No `100vh`; `svh`/`dvh` used correctly with `vh` fallback.
- [ ] `interactive-widget=resizes-content` set; bottom inputs stay above the keyboard.
- [ ] Safe-area insets applied; inputs ≥16px; tested on a real phone in both orientations.
