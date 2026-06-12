# CloaKey GitHub Pages SEO Strategy

This document outlines the Search Engine Optimization (SEO) specifications, metadata tags, structured schema configurations, indexing rules, and performance guidelines for the CloaKey documentation landing page.

---

## 🎯 Target Keywords & Search Intent

CloaKey targets users searching for ways to temporarily suppress input devices without locking their system session. 

### Core Keywords
-   **Primary:** `prevent accidental keyboard input`, `cat keyboard protection`, `keyboard lock software`, `mouse lock software`
-   **Secondary:** `developer workflow protection`, `child keyboard lock`, `toddler keyboard lock`, `windows mouse blocker`, `input hook blocker windows`
-   **Long-tail:** `how to lock keyboard while video is running`, `how to stop cat from typing on keyboard`, `lightweight keyboard click blocker`

---

## 🏷️ Meta Tag Configuration

To maximize Click-Through Rate (CTR) from search engine results pages (SERPs), configure the primary landing page header as follows:

```html
<!-- Primary Meta Tags -->
<title>CloaKey — Protect the workflow. Don't stop the workflow.</title>
<meta name="title" content="CloaKey — Protect the workflow. Don't stop the workflow.">
<meta name="description" content="CloaKey is a lightweight desktop utility that temporarily locks keyboard and mouse input on Windows. Keep terminal builds, videos, downloads, and AI agents running.">

<!-- Open Graph / Facebook -->
<meta property="og:type" content="website">
<meta property="og:url" content="https://cloakey.io/">
<meta property="og:title" content="CloaKey — Protect the workflow. Don't stop the workflow.">
<meta property="og:description" content="Temporarily suppress keyboard and mouse clicks without interrupting active tasks, rendering, or AI coding agents.">
<meta property="og:image" content="https://raw.githubusercontent.com/cloakey/cloakey/main/assets/cloakey_banner.png">

<!-- Twitter -->
<meta property="twitter:card" content="summary_large_image">
<meta property="twitter:url" content="https://cloakey.io/">
<meta property="twitter:title" content="CloaKey — Protect the workflow. Don't stop the workflow.">
<meta property="twitter:description" content="Temporarily suppress keyboard and mouse clicks without interrupting active tasks, rendering, or AI coding agents.">
<meta property="twitter:image" content="https://raw.githubusercontent.com/cloakey/cloakey/main/assets/cloakey_banner.png">
```

---

## 🤖 robots.txt Recommendation

Create a `robots.txt` at the root of the domain (`cloakey.io`) or the Pages root to allow complete indexing while excluding internal development artifact folder tracks:

```text
User-agent: *
Allow: /
Allow: /cli
Allow: /architecture
Allow: /distribution
Disallow: /assets/
Disallow: /target/
Disallow: /crates/

Sitemap: https://cloakey.io/sitemap.xml
```

---

## 🗺️ XML Sitemap Specification (`sitemap.xml`)

Structure the sitemap to ensure indexers crawl all primary document nodes:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
  <url>
    <loc>https://cloakey.io/</loc>
    <lastmod>2026-06-12</lastmod>
    <changefreq>weekly</changefreq>
    <priority>1.0</priority>
  </url>
  <url>
    <loc>https://cloakey.io/cli.html</loc>
    <lastmod>2026-06-12</lastmod>
    <changefreq>monthly</changefreq>
    <priority>0.8</priority>
  </url>
  <url>
    <loc>https://cloakey.io/architecture.html</loc>
    <lastmod>2026-06-12</lastmod>
    <changefreq>monthly</changefreq>
    <priority>0.7</priority>
  </url>
  <url>
    <loc>https://cloakey.io/distribution.html</loc>
    <lastmod>2026-06-12</lastmod>
    <changefreq>monthly</changefreq>
    <priority>0.7</priority>
  </url>
</urlset>
```

---

## 🛠️ Schema.org Structured Data (JSON-LD)

Embed this script in the landing page `<head>` to display rich search snippet visual cards on Google Search results:

```html
<script type="application/ld+json">
{
  "@context": "https://schema.org",
  "@type": "SoftwareApplication",
  "name": "CloaKey",
  "operatingSystem": "Windows 10, Windows 11",
  "applicationCategory": "UtilitiesApplication",
  "offers": {
    "@type": "Offer",
    "price": "0.00",
    "priceCurrency": "USD"
  },
  "description": "Lightweight desktop workflow protection utility that temporarily suppresses keyboard and mouse interaction while keeping active applications, terminals, and AI agents running.",
  "softwareVersion": "1.0.0",
  "downloadUrl": "https://github.com/cloakey/cloakey/releases",
  "featureList": [
    "Keyboard Lock",
    "Mouse Lock",
    "Full Input Protection",
    "Ghost Mode Mouse Movement",
    "Timed Auto-Unlock",
    "Emergency Bypass Override"
  ],
  "author": {
    "@type": "Organization",
    "name": "CloaKey Project",
    "url": "https://cloakey.io"
  }
}
</script>
```
