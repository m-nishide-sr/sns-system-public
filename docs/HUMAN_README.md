# 人間向け開発ドキュメント

## GitHub Codespacesを指定のDevContainerで開く方法

1. リポジトリの「<>Code▼」ボタンを押下
1. Codespacesタブで「＋」ボタンの右側にある「・・・」ボタンを押下
1. 「New with options...」ボタンを押下
1. 「Dev container configuration」のセレクトボックスで、開きたいDevContainerを指定する

## SAMのインストール手順

```bash
wget https://github.com/aws/aws-sam-cli/releases/latest/download/aws-sam-cli-linux-x86_64.zip
unzip aws-sam-cli-linux-x86_64.zip -d sam-installation
sudo ./sam-installation/install
```