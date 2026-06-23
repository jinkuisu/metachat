cfg = open('F:\\metacaht\\cef\\patch\\patch.cfg', 'rb').read()
old = b"  # AI agent layer\n  {\n    'name': 'chromiumfish_ai-agent_agent-layer',\n    'note': 'ChromiumFish: in-browser AI agent overlay',\n  },"
new = b"  # AI agent layer (requires actor_overlay resources, disabled by default)\n  {\n    'name': 'chromiumfish_ai-agent_agent-layer',\n    'condition': 'INCLUDE_FISH_AI_AGENT',\n    'note': 'ChromiumFish: in-browser AI agent overlay (set INCLUDE_FISH_AI_AGENT=1 to enable)',\n  },"
if old in cfg:
    cfg = cfg.replace(old, new, 1)
    open('F:\\metacaht\\cef\\patch\\patch.cfg', 'wb').write(cfg)
    print('Replaced')
else:
    print('Not found, trying CRLF')
    old2 = old.replace(b'\n', b'\r\n')
    new2 = new.replace(b'\n', b'\r\n')
    if old2 in cfg:
        cfg = cfg.replace(old2, new2, 1)
        open('F:\\metacaht\\cef\\patch\\patch.cfg', 'wb').write(cfg)
        print('Replaced with CRLF')
    else:
        idx = cfg.find(b'ai-agent_agent-layer')
        print('Found at:', idx)
        print(repr(cfg[idx-80:idx+100]))
