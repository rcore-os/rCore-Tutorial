#!/bin/sh
# 下面的 DEPLOY_DIR 目录将关联到 https://github.com/rcore-os/rCore-Tutorial-deploy 远程仓库
# 部署后可以通过 https://rcore-os.github.io/rCore-Tutorial-deploy 来访问
DEPLOY_DIR=../rCore-Tutorial-deploy/
CURRENT_DIR=$(pwd)
DEPLOY_DIR=$CURRENT_DIR/$DEPLOY_DIR

# 判断是否需要重新 clone 仓库
if [ -d "$DEPLOY_DIR" ]; then
    echo "$DEPLOY_DIR exists, resetting to remote master ..."
    cd $DEPLOY_DIR
else
    echo "$DEPLOY_DIR doesn't exist, cloning from remote ..."
    mkdir -p $DEPLOY_DIR
    cd $DEPLOY_DIR
    git init
    # 该脚本将强制通过 SSH 协议和 GitHub 通信
    git remote add origin git@github.com:rcore-os/rCore-Tutorial-deploy.git
fi

# 重置到 remote master
git fetch origin
git reset --hard origin/master

# 构建 GitBook 并复制
cd $CURRENT_DIR
gitbook build
cp -r _book/* $DEPLOY_DIR
cd $DEPLOY_DIR

# Commit 并 push
CURRENT_TIME=$(date +"%Y-%m-%d %H:%m:%S")
git add *
git commit -m "[Auto-deploy] Build $CURRENT_TIME"
git push origin master

# 返回当前目录
cd $CURRENT_DIR