import json,re,sys
d=json.load(open('all_apps.json'))
apps=list(d)
C={
 'undo':r'^(edit\.)?undo$|^history\.undo$|\.undo$',
 'redo':r'^(edit\.)?redo$|\.redo$',
 'history.list':r'history(\.list)?$|^edit\.history',
 'select.all':r'select(ion)?\.?all$|selectAll$',
 'select.none':r'deselect(All)?$|select(ion)?\.(none|clear|deselect)$|selectNone',
 'select.invert':r'invert(Selection)?$|select\.inverse|select\.invert',
 'selection.get':r'^(selection|select)\.(get|list|info|current)$|selection$',
 'selection.set':r'^(select|selection)\.(set|nodes|layers|objects|ids|byId|only)$|^layer\.select$|^clip\.select$|^photo(s)?\.select$|^node\.select$|^select$',
 'layer.list':r'^(layer|layers|node|nodes|track|tracks|object|objects|clip|clips)\.(list|tree|all)$|^document\.tree|^layers$',
 'layer.create':r'^(layer|node|object|track)\.(new|create|add)$|^layer\.new(Layer)?$|^node\.create$|^track\.(new|add|create)$',
 'delete':r'^(edit\.)?(delete|clear)$|^(layer|node|object|clip|track)\.(delete|remove)$',
 'duplicate':r'(^|\.)duplicate$|^edit\.duplicate',
 'group':r'(^|\.)group$|\.groupSelection$|^object\.group',
 'ungroup':r'(^|\.)ungroup$',
 'rename':r'(^|\.)rename(Layer)?$',
 'transform':r'(^|\.)(transform|move|translate|nudge|setPosition|position)$|transform\.(set|apply|move)$|freeTransform$',
 'rotate':r'(^|\.)rotate$|transform\.rotate',
 'scale':r'(^|\.)(scale|resize)$|transform\.scale',
 'arrange.front':r'bringToFront$|arrange\.front|toFront$',
 'align':r'(^|\.)align(Left|Center|Objects)?$|^align\.',
 'visibility':r'(^|\.)(hide|show|toggleVisibility|setVisible|visible|visibility|hideLayers|showLayers)$',
 'lock':r'(^|\.)(lock|toggleLock|setLocked|locked)$',
 'fill.set':r'(fill|color)\.set$|setFill(Color)?$|^paint\.(fill|setFill)|^fill\.',
 'stroke.set':r'stroke\.set$|setStroke$|^stroke\.(weight|color|width)',
 'opacity.set':r'(^|\.)(opacity|setOpacity)$',
 'blend.set':r'(^|\.)(blend|blendMode|setBlendMode)$',
 'prop.get/set':r'^(prop|property|properties|node|layer|object)\.(set|get|setProps|setProperties)$',
 'text.set':r'^(text|type|layer)\.(set|setText|setContent|content|edit)$|setText$',
 'file.new':r'^(file|document|project|session|package)\.new$',
 'file.open':r'^(file|document|project|session|package|catalog)\.open$',
 'file.save':r'^(file|document|project|session|package)\.save$',
 'file.saveAs':r'saveAs$',
 'file.close':r'^(file|document|project|session)\.close$',
 'file.import':r'^(file\.)?import$|^file\.import(File|Files|Media|Audio|Image)?$|^media\.import$',
 'file.export':r'^(file\.)?export$|^file\.export(As|Image|Png)?$|^export\.(run|file|image|png)$',
 'file.place':r'(^|\.)place(Embedded|Image|File)?$',
 'clipboard.copy':r'^(edit\.)?copy$',
 'clipboard.paste':r'^(edit\.)?paste$',
 'clipboard.cut':r'^(edit\.)?cut$',
 'view.zoomIn':r'zoom(\.)?In$|zoom\.in$',
 'view.zoomOut':r'zoom(\.)?Out$|zoom\.out$',
 'view.fit':r'(zoom|view)\.?(fit|toFit|fitAll|fitWindow|fitOnScreen|fitAllInWindow|zoomToFit)$|fitInWindow|zoomToFit|fitScreen',
 'view.actual':r'actualPixels|actualSize|zoom100|zoom\.reset|100',
 'time.set':r'^(time|playhead|transport|timeline)\.(set|goTo|goto|seek|setTime|locate|current)$|currentTime',
 'play':r'^(playback|transport|preview)\.(play|toggle|start|playPause)$',
 'marker.add':r'^(marker|markers|comp|timeline|sequence|edit)\.(add|addMarker|new|create)Marker?$|^marker\.(add|new|create)$|addMarker$',
 'doc.info':r'^(document|doc|project|session|comp|package)\.(info|inspect|summary|stats)$|^project\.summary$',
 'commands.list':r'^(engine|app|command|commands|help)\.(commands|list|listCommands)$|^engine\.commands$',
 'prefs.get':r'^(prefs|preferences|settings)\.(get|list|pages|read)$',
 'prefs.set':r'^(prefs|preferences|settings)\.(set|write|update)$',
 'batch':r'^(command|engine|commands|batch)\.(batch|run)$|^batch\.',
 'effect.apply':r'^(effect|effects|filter|fx)\.(apply|add)$',
 'mask.add':r'^mask\.(add|new|create)$|^layer\.(addMask|mask\.add)$',
 'crop':r'(^|\.)crop$',
 'flatten':r'flatten(Image)?$',
 'quit':r'(^|\.)(quit|exit)$',
}
out={}
for k,rx in C.items():
    r=re.compile(rx)
    out[k]={a:[x['id'] for x in d[a] if r.search(x['id'])][:4] for a in apps}
json.dump(out,open('vocab_matches.json','w'),indent=1)
for k,v in out.items():
    present=sum(1 for a in apps if v[a])
    distinct=len({i for a in apps for i in v[a][:1]})
    print(f"{k:16} apps={present:2} distinct_first_ids={distinct:2} ", '; '.join(f"{a[:4]}:{','.join(v[a][:2])}" for a in apps if v[a]))
