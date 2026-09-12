import {readFileSync} from 'node:fs';
import {createGatewayApp} from '/gateway/dist/app.js';
import {InMemoryGatewayStore} from '/gateway/dist/store.js';
const key=process.env.AI_GATEWAY_API_KEY || readFileSync(process.env.AI_GATEWAY_API_KEY_FILE,'utf8').trim();
const store=new InMemoryGatewayStore();
const now=new Date();
const intent=await store.createTopUpIntent('synthetic-flash-qa','1000000',now);
await store.recordTopUpSettlement({intentId:intent.id,transaction:'synthetic-memory-only',walletAddress:'synthetic-flash-qa',network:'solana:devnet',amountMicrousd:'1000000',settledAt:now});
const credential=await store.createApiKey('synthetic-flash-qa',now,'synthetic-pepper-at-least-thirty-two-chars');
let upstreamUsage,upstreamCalls=0;
let expectedCharge=0n;
const app=createGatewayApp({store,tokenPepper:'synthetic-pepper-at-least-thirty-two-chars',ambientApiKey:'unused',paymentMiddleware:(_q,_s,next)=>next(),vercel:{baseUrl:new URL('https://ai-gateway.vercel.sh/v1'),apiKey:key},fetch:async(url,init)=>{
 upstreamCalls++;
 if(upstreamCalls===1)return Response.json({error:"synthetic transient failure"},{status:503});
 const r=await fetch(url,init),text=await r.clone().text();
 console.log(JSON.stringify({upstreamCall:upstreamCalls,status:r.status}));
 for(const line of text.split('\n'))if(line.startsWith('data: ')&&line!=='data: [DONE]'){const item=JSON.parse(line.slice(6));if(item.usage){upstreamUsage=item.usage;expectedCharge+=BigInt(Math.ceil(item.usage.gateway_cost*1e6));}}
 return r;
}});

const server=app.listen(18083,'::',()=>console.log('FLASH_QA_GATEWAY_READY'));
// Synthetic balance and key exist only in this isolated process.
import {writeFileSync} from 'node:fs';
writeFileSync('/tmp/flash-qa-credential-v2',credential.key,{mode:0o600});
writeFileSync('/tmp/flash-qa-pid',String(process.pid));
const timer=setTimeout(()=>{server.close();server.closeAllConnections();},600000);
process.on('SIGTERM',async()=>{
 const account=await store.apiBalanceForWallet('synthetic-flash-qa');
 console.log(JSON.stringify({upstreamCalls,balance:account.balanceMicrousd,expectedChargeMicrousd:expectedCharge.toString(),chargedMicrousd:(1000000n-BigInt(account.balanceMicrousd)).toString(),productionLedgerWrites:0}));
 clearTimeout(timer);server.close();server.closeAllConnections();
});
