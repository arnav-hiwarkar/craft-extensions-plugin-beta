export const meta = {
  name: 'plugin-arch-v3',
  description: 'Convert the v2 plugin design into one checked spec, review it in diff-based shards until a capture-recapture stop rule fires, then render the owner deliverables',
  whenToUse: 'Finishing the craft plugin architecture from scratchpad/v2 with deterministic checks and measured convergence',
  phases: [
    { title: 'Extract', detail: 'split v2 into chunks; Sonnet turns each into spec records; anchor coverage gate' },
    { title: 'Incumbents', detail: 'top 30 plugins and SDK surface per incumbent app, with evidence' },
    { title: 'Merge', detail: 'deterministic merge; Opus adjudicates conflicts and near-duplicates' },
    { title: 'Map', detail: 'map incumbent rows to capabilities, in parallel with adjudication; missing ones become proposed' },
    { title: 'Gate', detail: 'deterministic checks; Sonnet fixes mechanical errors only' },
    { title: 'Review', detail: 'Opus reviewers on slices and diffs, Opus fixers write patches, stop rule' },
    { title: 'Render', detail: 'generated tables plus budgeted narrative; doc checks; fresh-eyes read' },
  ],
}

const V3 = '/private/tmp/claude-501/-Users-ash-Projects-plugin-beta/e1ff60be-d392-4350-8c1f-a3227f97c217/scratchpad/v3'
const S = V3 + '/scripts'
const V2 = '/private/tmp/claude-501/-Users-ash-Projects-plugin-beta/e1ff60be-d392-4350-8c1f-a3227f97c217/scratchpad/v2'
const RES = '/Users/ash/Projects/plugin-beta/craft-extensions/docs/plugins/architecture-2026-11-18'
const A = args || {}
const MAX_ROUNDS = A.maxRounds || 4
const SPLIT_WORDS = A.splitWords || 14000
const EXTRACT_MODEL = A.extractModel || 'sonnet'
const SKIP = new Set(A.skip || [])
const SHARDS = ['core', 'ui', 'data', 'sec', 'ec-render', 'ec-motion', 'pc-pixel', 'pc-doc', 'fc', 'plan']
const CLASSES = ['missing-capability', 'contradiction', 'interface', 'owner-rule', 'unsafe', 'crash', 'perf', 'missing-test', 'ambiguity', 'bloat']

const RULES = 'Owner rules (binding): full capability with nothing deferred; functionality first, with enforced permissions and third-party disclaimers; no plugin network in this build (an owner decision, recorded as DEC-COM-no-plugin-network, not a deferral); apps stay fast and never crash; one common architecture with behaviour in one place. Never touch git repositories. No code comments.'
const SPEC_RULES = `Spec format: JSON Lines in ${V3}/spec, one record per line; ${V3}/spec/schema.json defines kinds, required fields, reference fields, length caps and enums; ${V3}/spec/shards.json defines areas and text budgets. Ids are KIND-SCOPE-slug. Scope is COM for anything shared by all apps, SEC for security-only, EC/PC/FC for one app, AE/PS/FG for incumbent rows. The slug is the natural key lowercased: crate name, permission name, op name, contract name with @version, legacy id (k3, ec-t50, d-a, ws5), or a short kebab phrase for capabilities. Put legacy ids in "alias" and natural names in "name" so references resolve. Every record has area, status and src anchors like "common.md:120-134".`

const RUN = { type: 'object', properties: { exit: { type: 'integer' }, stdout: { type: 'string', maxLength: 8000 } }, required: ['exit', 'stdout'] }
const EXTRACT = { type: 'object', properties: { records: { type: 'integer' }, coverage: { type: 'number' }, uncovered: { type: 'integer' }, note: { type: 'string', maxLength: 300 } }, required: ['records', 'coverage', 'uncovered'] }
const COUNT = { type: 'object', properties: { written: { type: 'integer' }, errors_after: { type: 'integer' }, note: { type: 'string', maxLength: 400 } }, required: ['written'] }
const GAPS = {
  type: 'object',
  properties: {
    gaps: {
      type: 'array', maxItems: 12,
      items: {
        type: 'object',
        properties: {
          target: { type: 'string', maxLength: 90 },
          cls: { type: 'string', enum: CLASSES },
          sev: { type: 'string', enum: ['serious', 'minor'] },
          claim: { type: 'string', maxLength: 300 },
          evidence: { type: 'string', maxLength: 300 },
          fix: { type: 'string', maxLength: 300 },
        },
        required: ['target', 'cls', 'sev', 'claim', 'evidence', 'fix'],
      },
    },
    more: { type: 'boolean' },
  },
  required: ['gaps', 'more'],
}
const FIX = {
  type: 'object',
  properties: {
    accepted: { type: 'array', maxItems: 40, items: { type: 'string', maxLength: 120 } },
    accepted_minor: { type: 'array', maxItems: 40, items: { type: 'string', maxLength: 120 } },
    rejected: { type: 'array', maxItems: 40, items: { type: 'object', properties: { key: { type: 'string', maxLength: 120 }, reason: { type: 'string', maxLength: 200 } }, required: ['key', 'reason'] } },
    dupes: { type: 'array', maxItems: 20, items: { type: 'array', minItems: 2, maxItems: 4, items: { type: 'string', maxLength: 120 } } },
    errors_after: { type: 'integer' },
  },
  required: ['accepted', 'accepted_minor', 'rejected', 'dupes', 'errors_after'],
}
const ISSUES = {
  type: 'object',
  properties: {
    issues: { type: 'array', maxItems: 10, items: { type: 'object', properties: { file: { type: 'string' }, where: { type: 'string', maxLength: 120 }, problem: { type: 'string', maxLength: 300 }, fix: { type: 'string', maxLength: 300 }, factual: { type: 'boolean' } }, required: ['file', 'where', 'problem', 'fix', 'factual'] } },
  },
  required: ['issues'],
}

function lastJson(text) {
  const lines = (text || '').trim().split('\n').reverse()
  for (const l of lines) {
    try { return JSON.parse(l) } catch (e) { }
  }
  return null
}

async function sh(cmd, label, ph) {
  const r = await agent(`Run this exact shell command with the Bash tool, once, and do nothing else. Return its exit code and the last 8000 characters of its stdout.\n\ncd ${S} && ${cmd}`, { label, phase: ph, schema: RUN, model: 'haiku', effort: 'low' })
  if (!r) return { exit: -1, json: null, stdout: '' }
  return { exit: r.exit, json: lastJson(r.stdout), stdout: r.stdout }
}

const keyOf = g => g.target.toLowerCase().replace(/^new:/, '').trim() + '|' + g.cls

function canon(dupes) {
  const parent = {}
  const find = x => (parent[x] && parent[x] !== x ? (parent[x] = find(parent[x])) : x)
  for (const group of dupes || []) {
    const root = find(group[0])
    for (const k of group.slice(1)) parent[find(k)] = root
  }
  return find
}

if (!SKIP.has('extract')) {
  phase('Extract')
  const split = await sh(`python3 split.py ${SPLIT_WORDS}`, 'split v2', 'Extract')
  const chunks = split.json || []
  log(`${chunks.length} chunks, ${chunks.reduce((n, c) => n + c.words, 0)} words`)
  const APPS = [
    { code: 'AE', name: 'After Effects', area: 'ec-render or ec-motion', sources: `${V2}/effectcraft.md section 2 (grep -n "^## 2" to find it), ${V2}/final/flagships.md section 3, ${RES}/research/05-apps-motion.md` },
    { code: 'PS', name: 'Photoshop', area: 'pc-pixel or pc-doc', sources: `${V2}/photocraft.md benchmark sections (grep -n "Benchmark\\|incumbent\\|UXP" to find them), ${V2}/final/flagships.md section 4, ${RES}/research/06-apps-image-design.md` },
    { code: 'FG', name: 'Figma', area: 'fc', sources: `${V2}/figcraft.md section 1, ${V2}/final/flagships.md section 5, ${RES}/research/06-apps-image-design.md` },
  ]
  const incumbents = APPS.map(app => () => agent(
    `Build the incumbent traceability rows for ${app.name}. ${SPEC_RULES}\nRead only these sources first: ${app.sources}. Then load WebSearch with ToolSearch (query "select:WebSearch") and confirm the current most-used ${app.name} plugins and the SDK features they rely on.\nWrite ${V3}/extract/inc/${app.code}.jsonl containing:\n1. Exactly 30 INC rows (INC-${app.code}-slug): the most used ${app.name} plugins, ranked 1-30 by install base and influence, covering every major category. Fields: id, title, area (${app.area}), status "spec", rank, stmt (what it does for the user), evidence (the SDK features it uses, by exact symbol or API name), uses_api (API ids), caps [].\n2. API rows (API-${app.code}-slug) for every SDK surface item those plugins use plus any SDK feature a top plugin category depends on (60-120 rows): id, title, area, status "spec", stmt, evidence (exact symbol names or doc section), caps [].\nAPI rows whose only purpose is plugin network access get "decisions": ["DEC-COM-no-plugin-network"] and status "decided".\nRules: no invented plugins; evidence must name real SDK features; keep every field under the caps in schema.json. Return the number of rows written.`,
    { label: `incumbents:${app.code}`, phase: 'Incumbents', schema: COUNT, model: 'sonnet', effort: 'high' }))
  const extractAll = pipeline(chunks,
    c => agent(
      `Convert one chunk of the v2 plugin design into spec records. ${SPEC_RULES}\nRead ${V3}/spec/schema.json and ${V3}/spec/shards.json, then ${V3}/extract/src/${c.chunk}.md (each line is prefixed "doc:line| "). Write ${V3}/extract/frag/${c.chunk}.jsonl.\nRules:\n- Extract each normative fact once, as the most specific kind: crate or module -> CMP; canonical or adapter op -> OP; permission -> PERM; contract with invariants -> CON (props are checkable properties, budget is a number with a unit and machine class); owner or engineering decision -> DEC; test or acceptance criterion -> TST (measure is quantitative); quality scenario -> QAS; capability a plugin gets -> CAP; flagship -> FLG; workstream -> WS; defined term with banned synonyms -> TERM; owner requirement -> REQ.\n- A table row is a record. Rationale prose is dropped except in DEC.rationale.\n- Every record cites src anchors from this chunk. Do not invent facts.\n- References use ids by the naming rule; when you only know a legacy id or a name, write that string and the merge step resolves it.\n- Something the source calls deferred, later or not built becomes a CAP with status "proposed" whose stmt states the capability; never copy deferral wording.\n- Mark hot-path ops and contracts with "hot": true.\nWhen done, run: cd ${S} && python3 anchors.py ${c.chunk}\nIf coverage is below 0.9, read ${V3}/reports/anchors/${c.chunk}.json, look at the listed lines, add records for normative facts you missed or extend the src of the record that already holds them, and run anchors.py once more. Return the record count and final coverage.`,
      { label: `extract:${c.chunk}`, phase: 'Extract', schema: EXTRACT, model: EXTRACT_MODEL, effort: 'medium' }),
    (r, c) => (r && r.coverage < 0.9)
      ? agent(`Fill gaps in the extraction of ${c.chunk}. ${SPEC_RULES}\nRead ${V3}/reports/anchors/${c.chunk}.json (uncovered normative lines) and only those lines in ${V3}/extract/src/${c.chunk}.md. For each line that states a rule, number, permission, op, test or decision not already in ${V3}/extract/frag/${c.chunk}.jsonl, append a record (schema in ${V3}/spec/schema.json); for lines already covered, extend that record's src. Then run: cd ${S} && python3 anchors.py ${c.chunk}. Return the final coverage.`,
        { label: `fill:${c.chunk}`, phase: 'Extract', schema: EXTRACT, model: 'haiku', effort: 'medium' })
      : r)
  const [inc, ext] = await parallel([() => parallel(incumbents), () => extractAll])
  const low = (ext || []).filter(r => r && r.coverage < 0.85).length
  log(`extraction done; ${low} chunks below 0.85 anchor coverage; incumbent rows ${(inc || []).filter(Boolean).map(x => x.written).join('/')}`)
}

if (!SKIP.has('merge')) {
  phase('Merge')
  const m = await sh('python3 merge.py', 'merge', 'Merge')
  log('merge: ' + JSON.stringify(m.json))
  const PARTS = [
    { name: 'com-structure', re: '^(CMP|CON|OP)-COM-' },
    { name: 'security', re: '^(PERM|DEC)-(COM|SEC)-|^[A-Z]+-SEC-' },
    { name: 'effectcraft', re: '^[A-Z]+-(EC|AE)-' },
    { name: 'photocraft', re: '^[A-Z]+-(PC|PS)-' },
    { name: 'figcraft', re: '^[A-Z]+-(FC|FG)-' },
    { name: 'verification', re: '^(TST|QAS|REQ|FLG|WS|TERM|CAP)-COM-' },
  ]
  const adjudicate = () => parallel(PARTS.map(p => () => agent(
    `Adjudicate merge problems for ids matching the regex /${p.re}/. ${SPEC_RULES}\n${RULES}\nRead ${V3}/reports/merge.json: conflict_list (same id, different values), near_duplicates (capability pairs with similar titles), unresolved_refs (reference strings that match no id). Handle only items whose id (or first id) matches your regex; for unresolved refs, handle those whose most likely target matches it (grep ${V3}/spec/*.jsonl and ${V3}/extract/src/ for context).\nFor each conflict choose the correct value from the sources (src anchors point into ${V2}); merge near-duplicates into one record and delete the other; point unresolved refs at the right id, create the missing record with status "proposed", or drop the ref if it names nothing real.\nWrite patches to ${V3}/patches/r0/adjudicate/${p.name}.jsonl: full records, {"id":..., "_merge": true, field: value} for edits, {"id":..., "_delete": true}. Run: cd ${S} && python3 try_patch.py ${V3}/patches/r0/adjudicate/${p.name}.jsonl and remove anything it reports as introduced (C01, C02, C03). Return the number of patch lines.`,
    { label: `adjudicate:${p.name}`, phase: 'Merge', schema: COUNT, effort: 'high' })))
  const MAPS = [{ code: 'AE', app: 'EffectCraft' }, { code: 'PS', app: 'PhotoCraft' }, { code: 'FG', app: 'FigCraft' }]
  const mapAll = () => sh('python3 render.py digest', 'digest', 'Map').then(() => parallel(MAPS.map(x => () => agent(
    `Map ${x.code} incumbent rows to capabilities in ${x.app}. ${SPEC_RULES}\nRead the "## CAP" section and "OP names by area" in ${V3}/out/digest.md. Read the rows of ${V3}/spec/api.jsonl and ${V3}/spec/inc.jsonl whose ids start with API-${x.code}- or INC-${x.code}- (use grep).\nFor each API row set caps to the capability ids that let a ${x.app} plugin do the same thing at the same speed or better. Reuse COM capabilities for needs every app shares. If no capability exists, create one: CAP record with status "proposed", the right area, stmt saying what plugins need, via [], tests []. For each INC row, add caps only for needs its API rows do not cover (UI parity, performance class, data sizes, workflow).\nWrite ${V3}/patches/r0/map/${x.code}.jsonl ({"id":..., "_merge": true, "caps": [...]} for existing rows plus new CAP records). Run: cd ${S} && python3 try_patch.py ${V3}/patches/r0/map/${x.code}.jsonl and fix any introduced C01, C02 or C03 errors (C06 on proposed capabilities is expected). Return the number of patch lines.`,
    { label: `map:${x.code}`, phase: 'Map', schema: COUNT, model: 'sonnet', effort: 'high' }))))
  await parallel([adjudicate, mapAll])
  const ap = await sh(`python3 apply.py 0 adjudicate; python3 apply.py 0 map`, 'apply adjudication and mapping', 'Merge')
  log('adjudication and mapping applied: ' + JSON.stringify(ap.json))
}

if (!SKIP.has('gate')) {
  phase('Gate')
  for (let k = 1; k <= (A.gatePasses || 1); k++) {
    const c = await sh('python3 trace.py >/dev/null; python3 check.py', `check ${k}`, 'Gate')
    const by = (c.json && c.json.by_rule) || {}
    const mech = ['C01', 'C02', 'C03', 'C11', 'C15'].reduce((n, r) => n + (by[r] || 0), 0)
    log(`gate ${k}: ${JSON.stringify(by)}; mechanical ${mech}`)
    if (!mech) break
    await agent(
      `Fix mechanical spec errors only. ${SPEC_RULES}\nRead ${V3}/reports/check.json error_list and handle rules C01 (missing or malformed fields: fill from the record's src anchor in ${V2}), C02 (dangling refs: resolve by grep over names and aliases, or drop a ref that names nothing real), C03 (over a cap: shorten wording, keep every fact; over an area budget: remove duplicated wording first), C11 and C15. Do not touch design questions (C04-C14 other than C11) — reviewers handle those.\nWrite ${V3}/patches/r0/mech${k}/fix.jsonl in the patch format ({"id":..., "_merge": true, ...}, full records, or {"id":..., "_delete": true}), run cd ${S} && python3 try_patch.py ${V3}/patches/r0/mech${k}/fix.jsonl until it introduces nothing, and return the count.`,
      { label: `mechanical ${k}`, phase: 'Gate', schema: COUNT, model: 'sonnet', effort: 'medium' })
    await sh(`python3 apply.py 0 mech${k}`, `apply mech ${k}`, 'Gate')
  }
}

phase('Review')
const seen = new Set()
const lastAccepted = {}
let est = null
let rounds = 0
for (let r = A.startRound || 1; r <= MAX_ROUNDS; r++) {
  rounds = r
  const audit = SHARDS[(r * 3) % SHARDS.length]
  const prep = await sh(
    `python3 snap.py snap ${r} >/dev/null && python3 check.py >/dev/null; python3 trace.py >/dev/null; (for s in ${SHARDS.join(' ')} integration; do python3 slice.py $s ${r}; done; python3 slice.py ${audit} ${r} --full) | jq -sc 'map({key:(.path|split("/")|last|sub("^r[0-9]+-";"")|sub(".json$";"")), value:{t:.approx_tokens,c:.checks,g:.trace_gaps,ch:.changed}})|from_entries'`,
    `slices r${r}`, 'Review')
  const sizes = prep.json || {}
  const ranked = SHARDS.slice().sort((a, b) => (lastAccepted[b] || 0) - (lastAccepted[a] || 0))
  const active = SHARDS.filter(s => r === 1 || s === audit || !sizes[s] || sizes[s].ch > 0 || sizes[s].c > 0 || sizes[s].g > 0)
  const doubles = new Set(r === 1 ? SHARDS.filter((_, i) => i % 2 === 0) : [audit].concat(ranked.filter(s => active.includes(s) && s !== audit).slice(0, 2)))
  const skipped = SHARDS.filter(s => !active.includes(s))
  log(`round ${r}: ${active.length} shards active, doubles ${[...doubles].join(',')}${skipped.length ? `; unchanged and clean, not reviewed: ${skipped.join(',')}` : ''}`)
  const items = active.map(s => ({ s, stage: 'shards', double: doubles.has(s) })).concat([{ s: 'integration', stage: 'integration', double: false }])
  const ledgerRows = await pipeline(items,
    it => parallel(['A', 'B'].filter(t => t === 'A' || it.double).map(tag => () => agent(
      `You are reviewer ${tag} for shard "${it.s}", round ${r}. ${RULES}\nRead only ${V3}/reports/slices/r${r}-${it.s}.json: the shard lens, its records in full (round 1 or audit) or the diff since the last round with same-shard context, neighbour summaries, failing deterministic checks, traceability gaps, and the ledger of gaps already found. You may grep ${V3}/spec/*.jsonl for at most 8 ids you need in full. Do not read the v2 documents.${tag === 'B' ? ' Work through the slice in reverse: traceability gaps and neighbours first, then records from the last to the first.' : ''}\nReport defects the deterministic checks cannot see. Do not repeat ledger items or listed check failures.\nSerious means one of: an incumbent row (INC/API) cannot be matched at the same speed or better; two records contradict or an interface between components does not line up; an owner rule is broken (deferral, a restriction with no cause, missing disclaimer, plugin network, a permission that is not enforced); a plugin can crash, stall or bypass a permission in the app; a hot path has no budget or one that cannot be met. Everything else is minor.\nEach gap names an existing target id (or NEW:KIND-SCOPE-slug for a missing record), a class, a severity, a claim, evidence (ids, incumbent plugin and SDK symbol, or owner rule) and a one- or two-sentence fix. At most 12, most serious first; set more=true if you stopped with more to report.`,
      { label: `review:${it.s}:${tag}`, phase: 'Review', schema: GAPS, effort: 'high' }))),
    (revs, it) => {
      const [ra, rb] = revs
      const A_ = (ra && ra.gaps) || []
      const B_ = it.double ? ((rb && rb.gaps) || []) : null
      if ((ra && ra.more) || (rb && rb.more)) log(`review ${it.s} r${r}: reviewer hit the 12-gap cap; shard stays active next round`)
      const fresh = {}
      for (const g of A_.concat(B_ || [])) if (!seen.has(keyOf(g))) fresh[keyOf(g)] = fresh[keyOf(g)] || g
      const sz = sizes[it.s] || {}
      const work = Object.keys(fresh).length + (sz.c || 0) + (sz.g || 0)
      if (!work) return { it, A_, B_, fix: { accepted: [], accepted_minor: [], rejected: [], dupes: [], errors_after: 0 } }
      const list = Object.entries(fresh).map(([k, g]) => ({ key: k, target: g.target, cls: g.cls, sev: g.sev, claim: g.claim, evidence: g.evidence, fix: g.fix }))
      return agent(
        `You are the designer-fixer for shard "${it.s}", round ${r}. ${RULES}\n${SPEC_RULES}\nRead ${V3}/reports/slices/r${r}-${it.s}.json. You own this shard's records.\n1. Triage each reviewer gap below: accept as serious, accept as minor, or reject with a one-line reason (wrong, already handled, or asks for something no incumbent needs and no owner rule requires). When two keys describe one defect, list them together in dupes.\n2. Fix every failing check and traceability gap listed in the slice. These are not optional.\n3. Turn every record with status "proposed" in this shard into a designed record (realisers, permissions, tests with measurable acceptance, status "spec"), or merge it into an existing record and delete the duplicate.\nDesign rules: prefer editing an existing record over adding one; one fact lives in one record; keep the shard under its text budget in shards.json; tests carry a quantitative measure; never defer.\nWrite every change to ${V3}/patches/r${r}/${it.stage}/${it.s}.jsonl: full new records, {"id":..., "_merge": true, field: value} for edits, {"id":..., "_delete": true} for removals. Touch records of other areas only with _merge of reference fields when a fix needs it.\nRun cd ${S} && python3 try_patch.py ${V3}/patches/r${r}/${it.stage}/${it.s}.jsonl and iterate (at most 4 times) until nothing is introduced and this shard's errors are gone. Return triage keys exactly as given and errors_after for this shard.\n\nGaps:\n${JSON.stringify(list)}`,
        { label: `fix:${it.s}`, phase: 'Review', schema: FIX, effort: 'high' })
        .then(fix => ({ it, A_, B_, fix: fix || { accepted: [], accepted_minor: [], rejected: [], dupes: [], errors_after: -1 } }))
    })
  const shardsLedger = []
  for (const row of ledgerRows.filter(Boolean)) {
    const find = canon(row.fix.dupes)
    const mapG = gs => gs === null ? null : gs.map(g => ({ key: find(keyOf(g)), sev: g.sev, claim: g.claim }))
    const acc = [...new Set((row.fix.accepted || []).map(find))]
    const minor = [...new Set((row.fix.accepted_minor || []).map(find))]
    lastAccepted[row.it.s] = acc.length
    for (const g of row.A_.concat(row.B_ || [])) { seen.add(keyOf(g)); seen.add(find(keyOf(g))) }
    shardsLedger.push({ shard: row.it.s, A: mapG(row.A_), B: mapG(row.B_), accepted: acc, accepted_minor: minor })
  }
  const unreviewed = skipped.length
  const app = await sh(
    `cat > ${V3}/ledger/r${r}.json <<'LEDGER_EOF'\n${JSON.stringify({ round: r, unreviewed_weight: 0, skipped: unreviewed, shards: shardsLedger })}\nLEDGER_EOF\n(python3 apply.py ${r} shards; python3 apply.py ${r} integration) | jq -sc '{shards:.[0], integration:.[1]}'`,
    `apply r${r}`, 'Review')
  const conflicts = ((app.json && app.json.shards && app.json.shards.conflicts) || 0) + ((app.json && app.json.integration && app.json.integration.conflicts) || 0)
  if (conflicts) {
    await agent(
      `Patches from different shards in round ${r} edited the same records and were not applied. ${SPEC_RULES}\nRead ${V3}/reports/apply-r${r}-shards.json and apply-r${r}-integration.json (conflicts list the ids and patch files). For each id, read every patch line for it under ${V3}/patches/r${r}/, combine them into one correct record that keeps every intended fix, and write ${V3}/patches/r${r}/conflicts/merged.jsonl. Run cd ${S} && python3 try_patch.py on it until nothing is introduced. Return the count.`,
      { label: `conflicts r${r}`, phase: 'Review', schema: COUNT, model: 'sonnet', effort: 'high' })
    await sh(`python3 apply.py ${r} conflicts`, `apply conflicts r${r}`, 'Review')
  }
  const e = await sh(`python3 check.py >/dev/null; python3 trace.py >/dev/null; python3 estimate.py ${r}`, `estimate r${r}`, 'Review')
  est = e.json
  log(`round ${r}: D=${est && est.D} R_hi80=${est && est.R_hi80} tail=${est && est.tail_geometric} gates=${JSON.stringify(est && est.gates)} stop=${est && est.stop}`)
  if (est && est.stop) break
  if (r === MAX_ROUNDS) log(`round cap ${MAX_ROUNDS} reached without meeting the stop rule; the open estimate and gate failures go to the owner list`)
}

phase('Render')
const g = await sh(`(python3 trace.py >/dev/null; python3 render.py gen; python3 render.py digest) | jq -sc '{gen:.[0], digest:.[1]}'`, 'generate', 'Render')
const genWords = JSON.stringify((g.json && g.json.gen) || {})
const DOCS = `${V3}/out`
const writerCommon = `${RULES}\nYou write one owner-facing document from the single source of truth. Read ${DOCS}/digest.md first (every record's id and statement). Grep ${V3}/spec/*.jsonl for detail on at most 40 ids. Do not read the v2 documents. Generated tables live in ${DOCS}/gen/NAME.md and are inserted with {{gen:NAME}} placeholders; their word counts are ${genWords}. Required headings and word limits are in ${V3}/spec/schema.json under "deliverables"; the word limit includes inserted tables. Cite records by id in square brackets, at most once per paragraph, only where a reader would look it up. Plain, direct English; short sentences; explain the why once. No deferral words. After writing, run cd ${S} && python3 render.py fill and check your file's word count in the output; trim or expand to fit.`
await parallel([
  () => agent(`${writerCommon}\nWrite ${DOCS}/src/architecture.tmpl.md: the architecture an engineer builds from. Use {{gen:components}}, {{gen:contracts}}, {{gen:qas}} and {{gen:matrix}} where they belong. Each section says what exists, how the parts connect, and which decisions shaped it.`,
    { label: 'write:architecture', phase: 'Render', schema: COUNT, effort: 'high' }),
  () => agent(`${writerCommon}\nWrite ${DOCS}/src/how-plugins-work.tmpl.md: a deep but readable explainer for the owner of how plugins work in EffectCraft, PhotoCraft and FigCraft: how they use each application, how they are installed, granted, run, updated and removed, and what happens when they fail. Walk through one flagship per app end to end (pick from FLG records), naming the ops, permissions and contracts on the path. No tables except short ones you write yourself.`,
    { label: 'write:explainer', phase: 'Render', schema: COUNT, effort: 'high' }),
  () => agent(`${writerCommon}\nWrite three short templates: ${DOCS}/src/workstreams.tmpl.md (insert {{gen:workstreams}}; add the dependency structure from CMP deps and WS deps and the milestone sequence in prose), ${DOCS}/src/flagships.tmpl.md (one intro paragraph, then {{gen:flagships-EC}}, {{gen:flagships-PC}}, {{gen:flagships-FC}} under the required headings), ${DOCS}/src/decisions.tmpl.md (one paragraph on how the owner answers, then {{gen:decisions-open}} and {{gen:decisions-closed}}).`,
    { label: 'write:plan-docs', phase: 'Render', schema: COUNT, model: 'sonnet', effort: 'medium' }),
])
const FILL = `python3 render.py fill >/dev/null; mkdir -p ${DOCS}/final/reference; cp ${DOCS}/gen/perms.md ${DOCS}/gen/ops-reference.md ${DOCS}/final/reference/; python3 check.py --docs ${DOCS}/final --final`
let docCheck = await sh(FILL, 'doc check 1', 'Render')
for (let k = 2; k <= 3 && docCheck.exit !== 0; k++) {
  await agent(`Fix the document check failures in ${V3}/reports/check.json (rules D01-D06 for templates in ${DOCS}/src; spec rules C* by writing a patch to ${V3}/patches/r99/doc${k}/fix.jsonl and running cd ${S} && python3 try_patch.py on it). ${SPEC_RULES} Edit templates in ${DOCS}/src only, never ${DOCS}/final. Return the count of fixes.`,
    { label: `doc fix ${k}`, phase: 'Render', schema: COUNT, model: 'sonnet', effort: 'high' })
  docCheck = await sh(`python3 apply.py 99 doc${k} >/dev/null 2>&1; ${FILL}`, `doc check ${k}`, 'Render')
}
const read = await agent(`${RULES}\nYou are the owner's fresh-eyes reader. Read ${DOCS}/final/architecture.md, how-plugins-work.md, workstreams.md, flagships.md and decisions.md (not the reference folder). Report at most 10 issues that would stop the owner understanding how plugins work, how they use each app and how they are managed, or that look wrong on their face. Mark factual=true when the fix is a change to a spec record rather than wording.`,
  { label: 'fresh read', phase: 'Render', schema: ISSUES, effort: 'high' })
if (read && read.issues.length) {
  await agent(`Apply these reader issues. Wording issues: edit the templates in ${DOCS}/src. Factual issues: change the spec through a patch at ${V3}/patches/r99/reader/fix.jsonl (check with cd ${S} && python3 try_patch.py), never by editing generated text. ${SPEC_RULES}\n${JSON.stringify(read.issues)}`,
    { label: 'apply reader fixes', phase: 'Render', schema: COUNT, model: 'sonnet', effort: 'high' })
  docCheck = await sh(`python3 apply.py 99 reader >/dev/null 2>&1; python3 render.py gen >/dev/null; ${FILL}`, 'final doc check', 'Render')
}
return { rounds, estimate: est, docs: docCheck.json, reader_issues: read ? read.issues.length : null }
