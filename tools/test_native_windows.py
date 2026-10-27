"""Проверка настоящих окон Blackwire в отдельном временном профиле, без снимков экрана."""
from pathlib import Path
import argparse,json,subprocess,tempfile,os,time

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('binary',type=Path);args=p.parse_args()
    binary=args.binary.resolve();assert binary.is_file(),binary
    with tempfile.TemporaryDirectory(prefix='blackwire-native-windows-') as folder:
        env=os.environ.copy();env['WEBVIEW2_USER_DATA_FOLDER']=str(Path(folder)/'webview');env['XDG_CACHE_HOME']=str(Path(folder)/'cache');env['WEBKIT_DISABLE_DMABUF_RENDERER']='1'
        run=subprocess.run([str(binary),'--window-system-test','--data-dir',folder],timeout=80,env=env)
        time.sleep(2)
        report=Path(folder)/'native-window-report.json'
        assert report.is_file(),f'Window QA report missing; process exit {run.returncode}'
        data=json.loads(report.read_text(encoding='utf-8'));print(json.dumps(data,ensure_ascii=False,indent=2))
        assert run.returncode==0 and data['ok'],data
    print('Native window checks: OK')

if __name__=='__main__':main()
