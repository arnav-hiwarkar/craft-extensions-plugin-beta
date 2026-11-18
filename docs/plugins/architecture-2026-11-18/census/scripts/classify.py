import pathlib
import csv
import json
import re
import sys
from collections import Counter, defaultdict

C = str(pathlib.Path(__file__).resolve().parents[1])
APPS = 'effectcraft photocraft figcraft filmcraft anicraft lightcraft vectorcraft designcraft materialcraft texturecraft soundcraft'.split()

QUERY_FIELD = {
    'effectcraft': 'journal (bool; query! macro sets false)',
    'photocraft': 'journal (bool)',
    'figcraft': 'history (bool: edit = undo step)',
    'filmcraft': 'journal (bool; query!/query()/q() helpers)',
    'anicraft': 'mutates (bool: undo step)',
    'lightcraft': 'journal (bool; cmd!(query ..))',
    'vectorcraft': 'journal (bool; cmd!(query ..))',
    'designcraft': 'journal + undoable (cmd!(query|noundo ..))',
    'materialcraft': 'none',
    'texturecraft': 'none',
    'soundcraft': 'journal + undoable + view (cmd!(query|noundo|view ..))',
}

CLASSES = ['query', 'document edit', 'selection/view state', 'app state/preferences', 'file system', 'network', 'clipboard',
           'process/OS', 'collaboration/sharing', 'plugin/extension mgmt', 'undo/history', 'window/UI layout', 'long-running job',
           'AI/model download']

R = lambda p: re.compile(p, re.I)

RULES = [
    ('AI/model download', R(r'(model|models|voices?|weights)\.?(download|install|fetch|remove|delete)|download(model|voices|weights)|\bmodels?\.(download|install)|\.models\.')),
    ('plugin/extension mgmt', R(r'^(plugin|plugins|ext|extension|extensions|addon\.manage|vst3|clap|au)\.|\.(plugins?|vst3_plugins|rescan)\b|plugin\.(install|remove|enable|disable|reload|list|grant)|^engine\.(vst3|clap|au)')),
    ('collaboration/sharing', R(r'^(collab|comment|comments|share|sharing|live|presence|review|team|publish|cloud|multiplayer|feedback|annotation|annotations|facilitate|vote|cursor\.chat|session\.(join|host|leave))\.|\.(share|publish|unpublish|sendForReview|invite)\b|(^|\.)comment')),
    ('network', R(r'download|upload|\bfetch|http|\burl\b|openurl|geocode|placenames|onlineHelp|checkForUpdates|update\.check|\.web\b|stock|^font\.download|^library\.(sync|subscribe)|lens\.download|template\.install|marketplace|discord|getartcraft')),
    ('process/OS', R(r'(^|\.)reveal(InFinder|InExplorer|LogFile|Source|File|Folder|Original)?\b|InFinder|website|^help\.(website|forum|discord|community|github|report|docs|online|whatsnew|support|tutorials|feedback|bug)|executeFile|runScript|^script(s|ui)?\.|editOriginal|openSource|openExternal|openInExternal|launch\w*App|terminal|\bshell|addToPhotos|editIn(Photo|External|App)|openWith|systemBrowser|^macro\.|^actions?\.(play|run)|\.(runScript|evaluate|eval)\b|expression\.(eval|run)|^desktop\.|^control\.|app\.(quit|exit|restart)|file\.exit|^print\.(setup|preview|run|printers)|^file\.print|printOne|printers$')),
    ('clipboard', R(r'(^|\.)(copy|cut|paste|pasteInPlace|pasteInto|pasteSpecial|pasteBehind|pasteInFront|clipboard|duplicateToClipboard|copyMerged|copyAttributes|pasteAttributes|copyProperties|pasteProperties)\w*$|clipboard|^edit\.(copy|cut|paste)')),
    ('undo/history', R(r'(^|\.)(undo|redo|history|stepBackward|stepForward|revertTo|snapshot|snapshots|journal|purgeHistory|clearHistory)\b|^journal\.|^history\.|^edit\.(undo|redo)')),
    ('file system', R(r'^(file|files|io|doc|document|project|session|package|catalog|import|export|recent|link|links|relink|media|mediaBrowser|footage|asset|assets|library|preset|presets|watchFolder|render\.queue|batch)\.(open|save|import|export|close|new|place|relink|load|browse|recent|reveal|revert|watch|replace|collect|package|consolidate|backup|recover|restore|write|read|scan|add|remove|delete|navigate|roots)|\b(import|export|saveAs|save|open|load|relink|placeEmbedded|placeLinked|place|browse|exportAs|saveCopy|collectFiles|package|reloadFrom|linkFiles|embed)\w*\b|\.(open|save|load|import|export)\w*$|watchFolder|mediaBrowser|^file\.')),
    ('window/UI layout', R(r'^(window|workspace|workspaces|panel|panels|ui|tab|tabs|dock|toolbar|palette|frontend|app\.about|about|help)\.|\.(panel|panels|workspace|toolbar|showPanel|hidePanel|togglePanel|fullScreen|fullscreen|presentationMode|dock)\b|^app\.|toggle\w*Panel|^view\.(panel|show(?!Grid|Guides|Rulers)|hide)')),
    ('app state/preferences', R(r'^(prefs|preferences|settings|setting|shortcut|shortcuts|keyboard|keymap|theme|language|options|option|general|tool|tools|brush|brushes|swatches?|color\.settings|edit\.preferences|edit\.colorSettings|edit\.keyboardShortcuts|engine|audio\.device|device|midi\.input|transport\.(metronome|countIn|loop)|setup|config)\.|preferences|keyboardShortcuts|colorSettings|\.prefs\b|setPreference')),
    ('long-running job', R(r'^(render|bake|denoise|transcode|proxy|proxies|jobs?|queue|analy[sz]e|detect|tracker|motionTrack|trace|autotrace|stabilize|transcribe|transcript|speech|tts|merge\.(hdr|pano)|hdr|panorama|sceneDetect|scene_detect|contentFill|roto|segment|faces?|bounce|mixdown|export\.queue|renderQueue|render_queue)\.|\.(render|bake|analy[sz]e|detect|track(?!ing)|stabilize|transcribe|generate|process|bounce|mixdown|build|batch|denoise|upscale|superResolution|enhance)\w*$|contentAware|sceneEditDetection|autoReframe|morphCut|warpStabil|generativ')),
    ('selection/view state', R(r'(^|\.)(select|selection|deselect|selectAll|invertSelection|selectNone|reselect|focus)\w*\b|^(view|viewport|zoom|nav|navigate|go|goto|camera\.view|playhead|time\.(set|go)|transport|playback|scroll|timeline\.(zoom|scroll)|canvas|onion|display|preview|channel)\.|\.(zoom|zoomIn|zoomOut|fit|fitAll|fitWindow|actualPixels|scroll|pan|goTo|goto|seek|play|stop|pause|nextFrame|prevFrame|previousFrame|jumpTo|reveal)\w*\b|showGrid|showGuides|showRulers|snap|toggleGrid|toggleGuides|toggleRulers')),
]

QUERY_WORDS = R(r'^(list|get|info|inspect|find|count|query|describe|stats|status|state|summary|tree|bounds|measure|search|lookup|export(Data|Json)|histogram|metrics|report|check|validate|preflight|dump|all|current|resolve|compare|diff)\w*$|\.(list|get|info|tree|describe)\b')

DESTRUCTIVE_HARD = R(r'(^|\.)(delete|remove|uninstall|purge|discard|trash|wipe|reset|resetAll|revert|quit|exit|clearCache|clearRecent|clearHistory|purgeCache|deleteFile|deleteTemplate|deletePreset|overwrite|forget|restoreDefaults|factoryReset)\w*\b|clear(All|Recent|Cache|History|Prefs)|purge')
DESTRUCTIVE_SOFT = R(r'(^|\.)(delete|remove|clear|flatten|merge\w*|rasteri[sz]e|crop|trim|cut|reset|erase|collapse|bake|destroy|discard)\w*\b')


def classify(r):
    id_ = r['id']
    text = id_
    label = (r.get('label') or '')
    params = (r.get('params') or '')
    flags = set()
    primary = None
    for name, rx in RULES:
        if rx.search(text):
            primary = name
            break
    if primary is None and re.search(r'download|website|online|url', label, re.I):
        primary = 'network'
    path_key = re.search(r'(^|[{,|\s"(])(path|paths|file|files|dir|folder|outputPath|filePath|dest|destination)\??"?\s*[:?,=}|(]', params)
    if primary is None and path_key and re.search(r'\b(import|export|save|open|load|place|relink|write|read)', label, re.I):
        primary = 'file system'
    q = r.get('query')
    if primary not in ('network', 'process/OS', 'AI/model download', 'plugin/extension mgmt', 'clipboard', 'undo/history') and q is not False and QUERY_WORDS.search(id_.split('.')[-1] if '.' in id_ else id_) and (q or primary is None):
        if primary:
            flags.add(primary)
        primary = 'query'
    if primary is None:
        if q:
            primary = 'query'
        elif QUERY_WORDS.search(id_) and q is None:
            primary = 'query'
        else:
            primary = 'document edit'
    if primary == 'document edit' and q:
        primary = 'query'
    for name, rx in RULES:
        if name != primary and rx.search(text):
            flags.add(name)
    if path_key:
        flags.add('file system')
    if re.search(r'\burl\b|https?:', params, re.I):
        flags.add('network')
    always_hard = re.search(r'(^|\.)(quit|exit|uninstall|purge\w*|discard\w*|revert\w*|clearHistory|clearCache|clearRecent|deleteFile|factoryReset|restoreDefaults|forget\w*)$', id_, re.I) or re.search(r'^(prefs|preferences|settings|shortcuts|plugin|plugins|file|session|project|catalog|library|preset|presets|template|templates|workspace|window|app)\.(\w*[a-z](Delete|Remove|Reset|Clear)|(delete|remove|reset|clear))\w*$', id_)
    if always_hard or (DESTRUCTIVE_HARD.search(id_) and primary not in ('query', 'selection/view state', 'document edit', 'clipboard')):
        flags.add('dangerous/destructive')
    elif DESTRUCTIVE_HARD.search(id_) and primary == 'document edit':
        flags.add('doc-destructive (undoable)')
    elif DESTRUCTIVE_SOFT.search(id_) and primary == 'document edit':
        flags.add('doc-destructive (undoable)')
    if primary == 'file system' and re.search(r'export|save|write', id_, re.I):
        flags.add('fs.write')
    if primary == 'file system' and re.search(r'open|import|load|place|read|browse|relink', id_, re.I):
        flags.add('fs.read')
    flags.discard(primary)
    return primary, sorted(flags)


def params_shape(p):
    if p is None:
        return 'missing'
    t = p.strip()
    if t in ('', '{}', '{ }', '-'):
        return 'none'
    if t.startswith('{') or t.startswith('['):
        return 'keydoc'
    return 'prose'


def main():
    summary = {}
    for app in APPS:
        rows = json.load(open(f'{C}/stage_{app}.json'))
        out = []
        for r in rows:
            if app in ('materialcraft', 'texturecraft'):
                r['query'] = None
            if app in ('designcraft', 'soundcraft') and r.get('undoable') is None:
                v = r.get('variant')
                if r['ctor'] in ('flag!', 'toggle!', 'option!', 'radio!'):
                    v = 'noundo'
                r['undoable'] = v is None
                if app == 'soundcraft':
                    r['view'] = v == 'view'
            if app == 'figcraft' or app == 'anicraft':
                r['undoable'] = not r['query']
            cls, flags = classify(r)
            out.append(dict(
                app=app, id=r['id'], namespace=r['namespace'], label=r.get('label') or '', menu=r.get('menu') or '', shortcut=r.get('shortcut') or '',
                query_flag='' if r.get('query') is None else ('query' if r['query'] else 'edit'),
                undoable='' if r.get('undoable') is None else str(r['undoable']).lower(),
                view_only='' if r.get('view') is None else str(r['view']).lower(),
                params_shape=params_shape(r.get('params')), typed_schema=bool(r.get('schema')),
                params_doc=(r.get('params') or '').replace('\n', ' '), cls=cls, flags=';'.join(flags),
                source=f"{r.get('file')}:{r.get('line') or ''}", ctor=r.get('ctor'),
            ))
        out.sort(key=lambda x: x['id'])
        with open(f'{C}/{app}.csv', 'w', newline='') as f:
            w = csv.DictWriter(f, fieldnames=list(out[0].keys()))
            w.writeheader()
            w.writerows(out)
        json.dump(out, open(f'{C}/{app}.json', 'w'), indent=1)
        summary[app] = out
    json.dump({a: [{k: v for k, v in r.items() if k != 'params_doc'} for r in rows] for a, rows in summary.items()}, open(f'{C}/all_apps.json', 'w'))
    return summary


if __name__ == '__main__':
    s = main()
    hdr = ['app', 'total'] + CLASSES
    print('\t'.join(hdr))
    for app, rows in s.items():
        c = Counter(r['cls'] for r in rows)
        print('\t'.join([app, str(len(rows))] + [str(c.get(k, 0)) for k in CLASSES]))
    print()
    for flag in ['dangerous/destructive', 'doc-destructive (undoable)', 'file system', 'network', 'long-running job', 'process/OS', 'clipboard']:
        print(flag, {app: sum(1 for r in rows if flag in r['flags'].split(';') or r['cls'] == flag) for app, rows in s.items()})
