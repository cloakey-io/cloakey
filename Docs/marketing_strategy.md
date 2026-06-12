# Community Growth & Launch Strategy

This document details the community marketing strategy and campaign blueprints for launching CloaKey to developer hubs, productivity subreddits, and product launch networks.

---

## 📈 Hacker News (Show HN)

Hacker News appreciates transparency, lightweight design, and solving clear developer pain points.

### Guidelines
-   **Target URL:** Submit the GitHub repository directly: `https://github.com/cloakey/cloakey` (or the documentation site if it has the interactive terminal demo active).
-   **Title Styles:**
    -   `Show HN: CloaKey – A seatbelt for active developer workflows`
    -   `Show HN: CloaKey – Lock keyboard/mouse clicks, keep applications running`
-   **Timing:** Tuesdays or Wednesdays at 08:30 AM EST.
-   **Opening Comment (Maker Comment):**
    > "Hey HN,
    > 
    > I built CloaKey because I got tired of my cat stepping on my keyboard and aborting active build sessions, and my toddler accidentally hitting keys when I stepped away for a minute.
    > 
    > Existing OS screen-locking (like Win+L) suspends terminal visibility and can interrupt active background scripts. CloaKey blocks interaction hooks while keeping everything fully visible and active. It's written in Rust, uses native OS hooks, is 100% offline, and doesn't log keys.
    > 
    > Would love to hear your thoughts on configuration options, or ideas for the cross-platform macOS/Linux hooks in development!"

---

## 👽 Reddit Strategy

Reddit is highly skeptical of corporate promotion but supportive of open-source utilities that solve funny or annoying issues.

### Target Subreddits
1.  **r/rust:** Focus on the implementation. Detail how the Windows API mouse/keyboard hooks are coordinated in a safe Rust workspace with `ratatui` TUIs.
2.  **r/productivity:** Focus on workflow preservation. Explain the "Seatbelt for workflows" concept.
3.  **r/toddlers / r/cats:** Show the playful side. Present CloaKey as a simple tool to prevent "cat key attacks" or toddler typing accidents.
4.  **r/Windows11:** Highlight the integration (Winget package support, registry startup, tray config).

### Post Draft (r/rust)
-   **Title:** `CloaKey: A lightweight Windows input-hook blocker written in Rust`
-   **Body:**
    > "Hey r/rust, I wanted to share a tool I just finished building called CloaKey.
    > 
    > It intercepts and suppresses keyboard and mouse events using Windows hooks, allowing you to walk away from active terminal scripts, AI agents, or renders without risking accidental input. 
    > 
    > I chose Rust to ensure low-level Hook Proc latencies remain well under 1ms, keep memory under 20MB, and guarantee thread safety when dealing with win32 message loops. Check out the code here: [github.com/cloakey/cloakey](https://github.com/cloakey/cloakey)"

---

## 😸 Product Hunt Launch

Launch CloaKey as a developer and productivity tool.

### Assets Preparation
-   **Tagline:** *Protect your active workflow from cats, kids, and accidental key slips.*
-   **Gallery Visuals:**
    -   **Slide 1:** Clean product banner with tagline.
    -   **Slide 2 (GIF):** Interactive command demo (or a video of terminal execution).
    -   **Slide 3:** Table showing comparison between Screen Lock vs CloaKey.
-   **First Comment:** Focus on utility safety (offline-first, no logs) and request feedback on the V2 roadmap.

---

## 🐦 Twitter / X & Influencers

Focus on visual short-form video content showing:
1.  A terminal running a long, complex compiler build or script.
2.  A cat stepping directly onto the keyboard keys.
3.  Nothing happens — the terminal build completes successfully.
4.  Show the simple shortcut sequence (`Ctrl + Alt + Shift`) to restore control.
5.  *Caption:* "Protect the workflow. Don't stop the workflow. 🔒🐈 Built in Rust: github.com/cloakey/cloakey"
