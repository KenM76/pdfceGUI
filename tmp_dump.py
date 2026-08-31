import json, io, sys
out = json.load(io.open('tmp_findings.json', encoding='utf-8'))
w = io.open('tmp_findings.txt', 'w', encoding='utf-8')
for v in out:
    w.write('=' * 100 + '\n')
    w.write('ROW: %s | %s | conf %s\n' % (v.get('row'), v.get('verdict'), v.get('confidence')))
    w.write('SUMMARY: %s\n\n' % v.get('summary'))
    f = v.get('fix') or {}
    w.write('FIX SHAPE: %s\n' % f.get('shape'))
    w.write('FILES: %s\n' % ', '.join(f.get('files_to_touch') or []))
    w.write('RISKS: %s\n' % f.get('risks'))
    w.write('EFFORT: %s\n\n' % f.get('effort'))
    w.write('GENERALISATION: %s\n\n' % v.get('generalisation'))
    w.write('DRIVEN CHECK: %s\n' % v.get('driven_check'))
    w.write('ENGINE GAP: %s\n' % v.get('engine_gap'))
    w.write('EVIDENCE:\n')
    for e in (v.get('evidence') or []):
        w.write('  - %s :: %s\n' % (e.get('location'), e.get('what_it_shows')))
    w.write('\n')
w.close()
print('ok')
