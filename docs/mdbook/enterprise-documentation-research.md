# Enterprise Documentation Strategy for Lana Bank

## Executive Summary

Your codebase already uses **mdBook** (Rust ecosystem standard), has **2 GraphQL APIs** (Admin: 2,653 lines, Customer: 441 lines), **32 entities**, **43+ event types**, and follows event-sourcing with hexagonal architecture. You need a solution that:
- Supports versioning across releases
- Auto-generates API and event documentation
- Is embeddable and modular
- Projects enterprise credibility for B2B/BaaS

---

## Enterprise Documentation Landscape Analysis

### 1. Kubernetes (Reference Standard)
- **Framework**: [Hugo](https://github.com/kubernetes/website) with [Docsy theme](https://kubernetes.io/docs/home/)
- **Versioning**: Maintains docs for current + 4 previous versions via Git branches
- **API Docs**: Auto-generated from [OpenAPI spec using kubernetes-sigs/reference-docs](https://kubernetes.io/docs/contribute/generate-ref-docs/kubernetes-api/)
- **Key insight**: Migrated from Jekyll to Hugo for [build performance and multilingual support](https://kubernetes.io/blog/2018/05/05/hugo-migration/)

### 2. Stripe (Gold Standard for API Docs)
- **Framework**: Custom-built (highly tailored)
- **Versioning**: [Major releases twice yearly + monthly non-breaking releases](https://stripe.com/blog/api-versioning)
- **Key innovation**: [Auto-generated changelog, personalized docs per user's API version](https://docs.stripe.com/api/versioning)
- **Philosophy**: "APIs as infrastructure" - maintained compatibility since 2011

### 3. Mambu (Direct Banking Competitor)
- **Framework**: Custom portal at [api.mambu.com](https://api.mambu.com/) with [support.mambu.com](https://support.mambu.com/docs) for guides
- **Features**: OpenAPI specs available for SDK generation, [Configuration-as-Code documentation](https://support.mambu.com/docs/mambu-apis)
- **Versioning**: V1 (legacy) and V2 (recommended) maintained separately

### 4. Bitfinex (Crypto/Fintech Reference)
- **Framework**: [ReadMe.io](https://docs.bitfinex.com/) (managed platform)
- **Features**: Interactive API explorer, versioned (v1/v2), WebSocket + REST docs
- **Cost**: Enterprise ReadMe is ~$2,000/month

### 5. Plaid (Fintech Developer Experience Leader)
- **Framework**: Custom-built
- **Notable**: [Considered best-in-class alongside Stripe and Twilio](https://plaid.com/docs/)
- **Features**: Sandbox environments, comprehensive code samples across languages

---

## Framework Comparison Matrix

| Framework | Versioning | Auto-gen API | GraphQL Support | Embeddable | Enterprise Use | Learning Curve | Cost |
|-----------|------------|--------------|-----------------|------------|----------------|----------------|------|
| **[Docusaurus](https://docusaurus.io/)** | ✅ Native | Via plugins | Via plugins | ✅ | Meta, Azure, Figma | Medium (React/Node) | Free |
| **[Hugo](https://gohugo.io/)** | Manual | Via templates | Manual | ✅ | Kubernetes, DigitalOcean | High | Free |
| **[Antora](https://antora.org/)** | ✅ Native (multi-repo) | Via extensions | Limited | ✅ | Red Hat, Open Liberty, Magnolia | Medium-High | Free |
| **[mdBook](https://rust-lang.github.io/mdBook/)** | Manual | Limited | No | ✅ | Rust ecosystem | Low | Free |
| **[MkDocs + Material](https://squidfunk.github.io/mkdocs-material/)** | ✅ Via mike plugin | Via plugins | Limited | ✅ | Significant enterprise adoption | Low | Free (paid for sponsors) |
| **[ReadMe.io](https://readme.com/)** | ✅ Native | ✅ OpenAPI native | Limited | ⚠️ Hosted | Bitfinex, many APIs | Very Low | $2,000/mo Enterprise |
| **[Redocly](https://redocly.com/)** | ✅ Native | ✅ OpenAPI native | No | ✅ | Enterprise APIs | Low-Medium | Freemium |

---

## GraphQL Documentation Options

| Tool | Type | Features | Embedding | Maintenance |
|------|------|----------|-----------|-------------|
| **[SpectaQL](https://github.com/anvilco/spectaql)** | Static generator | Three-column layout, customizable | ✅ HTML | Active |
| **[DociQL](https://github.com/wayfair/dociql)** | Static generator | Beautiful output, embeddable | ✅ HTML | Wayfair maintained |
| **[Apollo Explorer](https://www.apollographql.com/docs/graphos/platform/explorer/embed)** | Interactive | Full IDE, embedded React/JS | ✅ React/JS/HTML | Apollo maintained |
| **[GraphDoc](https://github.com/2fd/graphdoc)** | Static generator | Simple, schema-based | ✅ HTML | Community |

---

## Versioning Approaches

### Branch-Based (Recommended for your case)
Used by: Kubernetes, Antora
- Each release gets a branch
- Docs evolve with code
- CI/CD builds version-specific sites

### Copy-Based
Used by: Docusaurus
- `docusaurus docs:version X.Y` copies entire docs folder
- Simple but can lead to drift

### API Version Headers
Used by: Stripe, Mambu
- Single doc set, version selector changes displayed content
- Best for pure API docs

---

## What Can Be Auto-Generated in Lana Bank

Based on codebase exploration:

| Artifact | Source | Tool Recommendation |
|----------|--------|---------------------|
| **GraphQL API Reference** | `schema.graphql` files | SpectaQL or DociQL → embed in main docs |
| **Entity Event Catalog** | `EsEvent` derives in `entity.rs` files | Custom Rust build script → JSON schema → Markdown |
| **Domain Model ERDs** | Entity relationships | Mermaid (already in your mdBook) |
| **Outbox Event Reference** | `LanaEvent` enum | Custom extractor using `schemars` (you have `json-schema` feature) |
| **Permission Matrix** | `CoreAccountingAction`, etc. | Extract from code → Markdown tables |
| **Job/Processor Catalog** | 46 job files | Glob + parse → documentation |

Your codebase already has `#[cfg_attr(feature = "json-schema", derive(JsonSchema))]` on many types - this is excellent for auto-generation.

---

## Recommended Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    Documentation Portal                          │
│                    (Docusaurus or Antora)                        │
├─────────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐   │
│  │   Guides     │  │   Domain     │  │    API Reference     │   │
│  │  (Markdown)  │  │   Model      │  │   (Auto-generated)   │   │
│  │              │  │  (Markdown   │  │                      │   │
│  │ • Quickstart │  │   + Mermaid) │  │ ┌─────────────────┐  │   │
│  │ • Tutorials  │  │              │  │ │ GraphQL Docs    │  │   │
│  │ • Concepts   │  │ • Entities   │  │ │ (SpectaQL/      │  │   │
│  │ • Workflows  │  │ • Events     │  │ │  DociQL embed)  │  │   │
│  │              │  │ • Jobs       │  │ └─────────────────┘  │   │
│  │              │  │ • Permissions│  │ ┌─────────────────┐  │   │
│  │              │  │              │  │ │ Event Catalog   │  │   │
│  │              │  │              │  │ │ (JSON Schema    │  │   │
│  └──────────────┘  └──────────────┘  │ │  generated)     │  │   │
│                                       │ └─────────────────┘  │   │
│                                       └──────────────────────┘   │
├─────────────────────────────────────────────────────────────────┤
│                     Version Selector (v1.0, v1.1, v2.0...)      │
└─────────────────────────────────────────────────────────────────┘
```

---

## Recommendations

### Option A: Docusaurus (Recommended for B2B credibility)

**Why:**
- Native versioning out of the box
- React-based = easy to embed interactive components (Apollo Explorer, custom visualizations)
- Used by major enterprises (Microsoft Azure, Figma, Shopify)
- Excellent search (Algolia integration)
- MDX support for rich content
- Active community, long-term Meta backing

**Implementation:**
```bash
# Structure
docs/
├── docusaurus.config.js
├── docs/                    # Main documentation (Markdown/MDX)
├── api/                     # Auto-generated API docs
│   ├── admin-graphql/       # SpectaQL output
│   └── customer-graphql/    # SpectaQL output
├── domain/                  # Auto-generated domain docs
│   ├── events/              # Event catalog from JSON schemas
│   └── entities/            # Entity documentation
├── versioned_docs/          # Previous versions
└── src/components/          # Custom React components
```

**Auto-generation pipeline:**
```yaml
# In CI/CD
- name: Generate GraphQL Docs
  run: npx spectaql ./spectaql-config.yml -t ./docs/api/admin-graphql

- name: Generate Event Catalog
  run: cargo run --bin doc-generator --features json-schema

- name: Build Docusaurus
  run: cd docs && npm run build
```

### Option B: Antora (If AsciiDoc preferred or multi-repo needed)

**Why:**
- Best-in-class multi-version, multi-repository support
- Used by Red Hat, Open Liberty, Magnolia (enterprise pedigree)
- AsciiDoc is more powerful than Markdown for technical docs
- Git branch-based versioning aligns with your release workflow

**Trade-off:** Steeper learning curve, requires AsciiDoc migration

### Option C: Hybrid - Keep mdBook + Add Docusaurus Shell

**Why:**
- Preserve existing mdBook investment
- Wrap with Docusaurus for versioning/navigation/search
- Embed mdBook output as iframe or static content

**Trade-off:** More complex build pipeline

---

## Implementation Roadmap

### Phase 1: Foundation
1. Choose framework (recommend Docusaurus)
2. Set up basic structure with version selector
3. Migrate existing mdBook content

### Phase 2: Auto-Generation
1. Integrate SpectaQL for GraphQL docs
2. Build Rust doc-generator for events/entities using existing `json-schema` feature
3. Set up CI/CD pipeline for doc generation

### Phase 3: Polish
1. Add interactive API explorer (Apollo Sandbox embed)
2. Implement search (Algolia)
3. Add analytics (understand what docs users need)

### Phase 4: Enterprise Features
1. Private docs section (for bank partners)
2. PDF export capability
3. Changelog automation

---

## Summary Table

| Requirement | Recommended Solution |
|-------------|---------------------|
| **Primary Framework** | Docusaurus 3.x |
| **Versioning** | Native Docusaurus versioning |
| **GraphQL API Docs** | SpectaQL (static) + Apollo Sandbox (interactive) |
| **Event/Entity Docs** | Custom Rust generator → JSON Schema → Markdown |
| **Domain Diagrams** | Mermaid (keep existing) |
| **Search** | Algolia DocSearch (free for open source) |
| **Hosting** | Vercel, Netlify, or GitHub Pages |
| **CI/CD** | GitHub Actions (already in use) |

---

## Sources

- [Kubernetes Website Repository](https://github.com/kubernetes/website)
- [Kubernetes Hugo Migration Blog](https://kubernetes.io/blog/2018/05/05/hugo-migration/)
- [Stripe API Versioning](https://stripe.com/blog/api-versioning)
- [Stripe Versioning Docs](https://docs.stripe.com/api/versioning)
- [Mambu APIs Overview](https://support.mambu.com/docs/mambu-apis)
- [Mambu API Reference](https://api.mambu.com/)
- [Bitfinex API Documentation](https://docs.bitfinex.com/)
- [Antora Documentation](https://antora.org/)
- [Antora Versioning Methods](https://docs.antora.org/antora/latest/content-source-versioning-methods/)
- [Docusaurus](https://docusaurus.io/)
- [SpectaQL GraphQL Documentation Generator](https://github.com/anvilco/spectaql)
- [DociQL by Wayfair](https://github.com/wayfair/dociql)
- [Apollo Explorer Embedding](https://www.apollographql.com/docs/graphos/platform/explorer/embed)
- [ReadMe.io Pricing](https://readme.com/pricing)
- [Redocly](https://redocly.com/)
- [Redoc GitHub](https://github.com/Redocly/redoc)
- [mdBook](https://rust-lang.github.io/mdBook/)
- [MkDocs Material](https://squidfunk.github.io/mkdocs-material/)
- [Plaid Documentation](https://plaid.com/docs/)
- [Hugo](https://gohugo.io/)
