安装Anchor,版本使用0.29.0
https://www.anchor-lang.com/docs/installation


安装完毕后，控制台运行

修改项目中Anchor.toml文件的 wallet = "/home/pluto/.config/solana/id.json" 为自己的钱包地址，并确保钱包中有足够的SOL
可以使用https://faucet.solana.com/给自己的钱包进行空投

安装必须的包：yarn install

构建项目：Anchor Build

部署项目：Anchor Deploy

测试项目：Anchor run test

取消it("mint")的注释，为自己的钱包中创建一个NFT，控制台中会log出NFT相关地址

修改nft-staking文件中mintAddress、owner、nftEditionKey为自己NFT地址

取消it("stake")的注释，进行质押测试

取消it("unstake")的注释，进行解除质押测试


