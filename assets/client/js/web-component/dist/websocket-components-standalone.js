import{b as pe,a as g,f as W,t as d,i as r,c as f,g as j,h as H,m as Z,S as m,d as De,F as Me}from"./index-iYDJ9E20.js";var Ue=d("<span>"),Be=d("<span class=user-count>(<!> user<!>)"),ze=d(`<div><style>
        .status-indicator {
          width: 12px;
          height: 12px;
          border-radius: 50%;
          border: 1px solid rgba(0, 0, 0, 0.1);
          transition: all 0.3s ease;
          position: relative;
        }

        .status-indicator.disconnected {
          background-color: #ef4444;
          box-shadow: 0 0 4px rgba(239, 68, 68, 0.3);
        }

        .status-indicator.connecting {
          background-color: #f59e0b;
          box-shadow: 0 0 4px rgba(245, 158, 11, 0.3);
          animation: pulse 1.5s infinite;
        }

        .status-indicator.connected {
          background-color: #10b981;
          box-shadow: 0 0 4px rgba(16, 185, 129, 0.3);
        }

        .status-indicator.error {
          background-color: #dc2626;
          box-shadow: 0 0 4px rgba(220, 38, 38, 0.5);
          animation: blink 1s infinite;
        }

        @keyframes pulse {
          0%, 100% {
            opacity: 1;
            transform: scale(1);
          }
          50% {
            opacity: 0.7;
            transform: scale(1.1);
          }
        }

        @keyframes blink {
          0%, 50% {
            opacity: 1;
          }
          51%, 100% {
            opacity: 0.3;
          }
        }

        .status-text {
          color: #374151;
          font-weight: 500;
        }

        .status-text.disconnected {
          color: #dc2626;
        }

        .status-text.connecting {
          color: #d97706;
        }

        .status-text.connected {
          color: #059669;
        }

        .status-text.error {
          color: #dc2626;
        }

        .user-count {
          color: #6b7280;
          font-size: 12px;
          margin-left: 4px;
        }
      </style><div>`),u=(i=>(i.Disconnected="disconnected",i.Connecting="connecting",i.Connected="connected",i.Error="error",i))(u||{});const ge=i=>{const[_,q]=g(Date.now()),C=()=>i.status??"disconnected",h=()=>i.showText??!0,B=()=>i.userCount??0,z=()=>i.showUserCount??!1,P=()=>i.compact??!1;W(()=>{const c=C();q(Date.now());const D=new CustomEvent("status-change",{detail:{status:c,timestamp:_()},bubbles:!0});setTimeout(()=>{const $=document.querySelector("websocket-status");$&&$.dispatchEvent(D)},0)});const G=()=>{switch(C()){case"disconnected":return"Offline";case"connecting":return"Connecting...";case"connected":return"Online";case"error":return"Connection Error";default:return"Unknown"}},R=()=>`status-indicator ${C()}`,E=()=>`status-text ${C()}`;return(()=>{var c=ze(),D=c.firstChild,$=D.nextSibling;return c.style.setProperty("display","inline-flex"),c.style.setProperty("align-items","center"),c.style.setProperty("gap","8px"),c.style.setProperty("font-family",'-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif'),c.style.setProperty("font-size","14px"),r(c,f(m,{get when(){return Z(()=>!!h())()&&!P()},get children(){var p=Ue();return r(p,G),j(()=>H(p,E())),p}}),null),r(c,f(m,{get when(){return Z(()=>!!(z()&&B()>0))()&&!P()},get children(){var p=Be(),J=p.firstChild,L=J.nextSibling,O=L.nextSibling,w=O.nextSibling;return w.nextSibling,r(p,B,L),r(p,()=>B()!==1?"s":"",w),p}}),null),j(()=>H($,R())),c})()};pe("websocket-status",{status:"disconnected",showText:!0,userCount:0,showUserCount:!1,compact:!1},ge);var Pe=d("<button>Disconnect"),Le=d("<div class=error-message>"),Oe=d("<div class=debug-log>"),Te=d("<div class=upload-progress>"),Ae=d('<div><div class=upload-controls><div class=file-input-wrapper><input type=file id=file-input class=file-input multiple><label for=file-input><svg class=upload-icon fill=none stroke=currentColor viewBox="0 0 24 24"><path stroke-linecap=round stroke-linejoin=round stroke-width=2 d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12"></path></svg></label></div><div class=upload-hint>'),Fe=d(`<div><style>
        .container {
          padding: 16px;
          border: 1px solid #e5e7eb;
          border-radius: 8px;
          background: #f9fafb;
        }

        .header {
          display: flex;
          align-items: center;
          justify-content: space-between;
          margin-bottom: 16px;
        }

        .title {
          font-size: 18px;
          font-weight: 600;
          color: #111827;
        }

        .controls {
          display: flex;
          gap: 8px;
        }

        button {
          padding: 6px 12px;
          border: 1px solid #d1d5db;
          border-radius: 6px;
          background: white;
          color: #374151;
          font-size: 14px;
          cursor: pointer;
          transition: all 0.2s;
        }

        button:hover {
          background: #f3f4f6;
          border-color: #9ca3af;
        }

        button:disabled {
          opacity: 0.5;
          cursor: not-allowed;
        }

        button.primary {
          background: #3b82f6;
          color: white;
          border-color: #3b82f6;
        }

        button.primary:hover {
          background: #2563eb;
          border-color: #2563eb;
        }

        .status-section {
          margin-bottom: 16px;
        }

        .debug-log {
          background: #1f2937;
          color: #f3f4f6;
          padding: 12px;
          border-radius: 6px;
          font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
          font-size: 12px;
          max-height: 300px;
          overflow-y: auto;
          white-space: pre-wrap;
          word-break: break-all;
        }

        .media-blobs {
          margin-top: 16px;
        }

        .media-blob {
          padding: 12px;
          border: 1px solid #e5e7eb;
          border-radius: 6px;
          margin-bottom: 8px;
          background: white;
        }

        .media-blob-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 8px;
        }

        .media-blob-id {
          font-family: monospace;
          font-size: 12px;
          color: #6b7280;
        }

        .media-blob-info {
          font-size: 14px;
          color: #374151;
        }

        .media-blob-meta {
          font-size: 12px;
          color: #6b7280;
          margin-top: 4px;
        }

        .empty-state {
          text-align: center;
          color: #6b7280;
          font-style: italic;
          padding: 32px;
        }

        .error-message {
          background: #fef2f2;
          border: 1px solid #fecaca;
          color: #dc2626;
          padding: 12px;
          border-radius: 6px;
          margin-bottom: 16px;
        }

        .file-upload-section {
          margin-top: 16px;
          padding: 16px;
          border: 2px dashed #d1d5db;
          border-radius: 8px;
          background: #f9fafb;
          transition: all 0.2s;
        }

        .file-upload-section.drag-over {
          border-color: #3b82f6;
          background: #eff6ff;
        }

        .file-upload-section.uploading {
          border-color: #10b981;
          background: #ecfdf5;
        }

        .upload-controls {
          display: flex;
          flex-direction: column;
          align-items: center;
          gap: 12px;
        }

        .file-input-wrapper {
          position: relative;
          overflow: hidden;
          display: inline-block;
        }

        .file-input {
          position: absolute;
          left: -9999px;
          opacity: 0;
        }

        .file-input-label {
          display: inline-flex;
          align-items: center;
          gap: 8px;
          padding: 10px 20px;
          background: #3b82f6;
          color: white;
          border-radius: 6px;
          cursor: pointer;
          font-size: 14px;
          font-weight: 500;
          transition: background 0.2s;
        }

        .file-input-label:hover {
          background: #2563eb;
        }

        .file-input-label:disabled {
          background: #9ca3af;
          cursor: not-allowed;
        }

        .upload-hint {
          color: #6b7280;
          font-size: 14px;
          text-align: center;
          margin: 8px 0;
        }

        .upload-progress {
          color: #374151;
          font-size: 14px;
          font-weight: 500;
          text-align: center;
          padding: 8px;
          background: #f3f4f6;
          border-radius: 4px;
          margin-top: 8px;
        }

        .upload-icon {
          display: inline-block;
          width: 16px;
          height: 16px;
        }
      </style><div class=container><div class=header><h2 class=title>WebSocket Handler</h2><div class=controls><button>Ping</button><button>Get Media Blobs</button></div></div><div class=status-section></div><div class=media-blobs><h3>Media Blobs (<!>)`),Ie=d("<button class=primary>Connect"),Ne=d('<div class=empty-state>No media blobs received yet. Click "Get Media Blobs" to fetch from server.'),We=d("<br>"),je=d("<div class=media-blob><div class=media-blob-header><div class=media-blob-id></div><div class=media-blob-info> • </div></div><div class=media-blob-meta>SHA256: <br>Client: <br>Path: <br>Created: ");const He=i=>{const _=()=>i.websocketUrl??"",q=()=>i.autoConnect??!0,C=()=>i.showDebugLog??!0,[h,B]=g(u.Disconnected),[z,P]=g(null),[G,R]=g([]),[E,c]=g([]),[D,$]=g(""),[p,J]=g(0),[L,O]=g(!1),[w,ee]=g(!1),[te,S]=g("");W(()=>{q()&&_()&&K()});const a=(e,...t)=>{const o=new Date().toLocaleTimeString(),l=t.length>0?`[${o}] ${e}: ${JSON.stringify(t,null,2)}`:`[${o}] ${e}`;R(b=>[...b.slice(-99),l]),console.log("[WebSocketHandler]",e,...t)},M=e=>{if(h()!==e){B(e),a(`Status changed to: ${e}`);const t=new CustomEvent("status-change",{detail:{status:e},bubbles:!0});setTimeout(()=>{const o=document.querySelector("websocket-handler");o&&o.dispatchEvent(t)},0)}},x=e=>{$(e),a(`Error: ${e}`)},oe=()=>{$("")},K=()=>{if(!_()){x("WebSocket URL not provided");return}if(z()?.readyState===WebSocket.OPEN){a("Already connected");return}oe(),M(u.Connecting),a(`Connecting to ${_()}`);try{const e=new WebSocket(_());P(e),fe(e)}catch(e){x(`Connection failed: ${e}`),M(u.Error)}},Q=()=>{a("Disconnecting...");const e=z();e&&(e.close(1e3,"Client disconnect"),P(null)),M(u.Disconnected)},fe=e=>{e.onopen=()=>{a("Connected successfully"),M(u.Connected),oe()},e.onclose=t=>{a("Connection closed",{code:t.code,reason:t.reason}),M(u.Disconnected)},e.onerror=t=>{a("Socket error",t),M(u.Error),x("Connection error occurred")},e.onmessage=t=>{me(t.data)}},me=e=>{a("Received message");try{const t=JSON.parse(e);switch(a("Parsed message type:",t.type),t.type){case"Welcome":a("Welcome received",t.data);break;case"Pong":a("Pong received");break;case"MediaBlobs":{const o=t.data;a("Media blobs received:",{count:o?.blobs?.length||0,total_count:o?.total_count}),c(o?.blobs||[]);const l=new CustomEvent("media-blobs-received",{detail:{blobs:E(),totalCount:o?.total_count},bubbles:!0});setTimeout(()=>{const b=document.querySelector("websocket-handler");b&&b.dispatchEvent(l)},0);break}case"MediaBlob":{const o=t.data;a("Single media blob received:",{id:o?.blob?.id,size:o?.blob?.size,mime:o?.blob?.mime});const l=new CustomEvent("media-blob-received",{detail:{blob:o?.blob},bubbles:!0});setTimeout(()=>{const b=document.querySelector("websocket-handler");b&&b.dispatchEvent(l)},0);break}case"Error":{a("Error message received",t.data);const o=t.data;x(o?.message||"Server error");break}case"ConnectionStatus":{a("Connection status update",t.data);const o=t.data;J(o?.user_count||0);break}default:a("Unknown message type:",t.type)}}catch(t){a("Failed to parse message",{error:t instanceof Error?t.toString():String(t),messageLength:e.length}),x(`Message parse error: ${t}`)}},F=e=>{const t=z();if(!t||t.readyState!==WebSocket.OPEN)return x("Cannot send message: not connected"),!1;try{const o=JSON.stringify(e);return t.send(o),e.type==="UploadMediaBlob"?a("Sent UploadMediaBlob message",{type:e.type,blob_id:e.data?.blob?.id,blob_size:e.data?.blob?.size,blob_mime:e.data?.blob?.mime,blob_sha256:e.data?.blob?.sha256?.substring(0,8)+"..."}):a("Sent message",e),!0}catch(o){return x(`Send error: ${o}`),!1}},ne=()=>F({type:"Ping"}),re=(e,t)=>F({type:"GetMediaBlobs",data:{limit:e,offset:t}}),he=e=>F({type:"GetMediaBlob",data:{id:e}}),ae=e=>F({type:"UploadMediaBlob",data:{blob:e}}),xe=async e=>{const t=await e.arrayBuffer(),o=await crypto.subtle.digest("SHA-256",t);return Array.from(new Uint8Array(o)).map(b=>b.toString(16).padStart(2,"0")).join("")},ve=async e=>{const t=await xe(e),o=await e.arrayBuffer(),l=Array.from(new Uint8Array(o));return{id:crypto.randomUUID(),data:l,sha256:t,size:e.size,mime:e.type||"application/octet-stream",source_client_id:"web-component",local_path:e.name,metadata:{originalName:e.name,lastModified:e.lastModified,uploadedAt:new Date().toISOString()},created_at:new Date().toISOString(),updated_at:new Date().toISOString()}},V=async e=>{if(e){ee(!0),S(`Preparing ${e.name}...`);try{a(`Starting upload for file: ${e.name} (${e.size} bytes)`),S("Calculating SHA256...");const t=await ve(e);if(S("Uploading to server..."),a("Uploading blob:",{id:t.id,size:t.size,mime:t.mime,sha256:t.sha256.substring(0,8)+"..."}),ae(t))S(`✅ ${e.name} uploaded successfully!`),a(`File upload successful: ${e.name}`),setTimeout(()=>S(""),3e3);else throw new Error("Failed to send upload message")}catch(t){const o=`Upload failed: ${t instanceof Error?t.message:String(t)}`;S(`❌ ${o}`),x(o),a("Upload error",t),setTimeout(()=>S(""),5e3)}finally{ee(!1)}}},ye=e=>{const t=e.target,o=t.files;o&&o.length>0&&Array.from(o).forEach(V),t.value=""},$e=e=>{e.preventDefault(),O(!0)},we=e=>{e.preventDefault(),O(!1)},Se=e=>{e.preventDefault(),O(!1);const t=e.dataTransfer?.files;t&&t.length>0&&Array.from(t).forEach(V)},ke=()=>{const e=document.querySelector("websocket-handler");e&&Object.assign(e,{ping:ne,getMediaBlobs:re,getMediaBlob:he,uploadMediaBlob:ae,uploadFile:V,connect:K,disconnect:Q})};W(()=>{setTimeout(ke,0)});const _e=e=>{if(!e)return"Unknown size";const t=["B","KB","MB","GB"];let o=e,l=0;for(;o>=1024&&l<t.length-1;)o/=1024,l++;return`${o.toFixed(1)} ${t[l]}`};return W(()=>Q),(()=>{var e=Fe(),t=e.firstChild,o=t.nextSibling,l=o.firstChild,b=l.firstChild,se=b.nextSibling,X=se.firstChild,ie=X.nextSibling,le=l.nextSibling,T=le.nextSibling,ce=T.firstChild,Ce=ce.firstChild,de=Ce.nextSibling;return de.nextSibling,e.style.setProperty("display","block"),e.style.setProperty("font-family",'-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif'),X.$$click=ne,ie.$$click=()=>re(),r(se,f(m,{get when(){return h()===u.Connected},get fallback(){return(()=>{var n=Ie();return n.$$click=K,n})()},get children(){var n=Pe();return n.$$click=Q,n}}),null),r(le,()=>ge({status:h(),userCount:p(),showUserCount:!0,showText:!0,compact:!1})),r(o,f(m,{get when(){return D()},get children(){var n=Le();return r(n,D),n}}),T),r(o,f(m,{get when(){return C()},get children(){var n=Oe();return r(n,()=>G().join(`
`)),n}}),T),r(o,f(m,{get when(){return h()===u.Connected},get children(){var n=Ae(),v=n.firstChild,y=v.firstChild,U=y.firstChild,k=U.nextSibling;k.firstChild;var Y=y.nextSibling;return n.addEventListener("drop",Se),n.addEventListener("dragleave",we),n.addEventListener("dragover",$e),U.addEventListener("change",ye),r(k,()=>w()?"Uploading...":"Choose Files",null),r(Y,()=>L()?"Drop files here to upload":"Drag & drop files here or click to select"),r(v,f(m,{get when(){return te()},get children(){var s=Te();return r(s,te),s}}),null),j(s=>{var I=`file-upload-section ${L()?"drag-over":""} ${w()?"uploading":""}`,A=w(),N=`file-input-label ${w()?"disabled":""}`;return I!==s.e&&H(n,s.e=I),A!==s.t&&(U.disabled=s.t=A),N!==s.a&&H(k,s.a=N),s},{e:void 0,t:void 0,a:void 0}),n}}),T),r(ce,()=>E().length,de),r(T,f(m,{get when(){return E().length>0},get fallback(){return Ne()},get children(){return f(Me,{get each(){return E()},children:n=>(()=>{var v=je(),y=v.firstChild,U=y.firstChild,k=U.nextSibling,Y=k.firstChild,s=y.nextSibling,I=s.firstChild,A=I.nextSibling,N=A.nextSibling,ue=N.nextSibling,Ee=ue.nextSibling,be=Ee.nextSibling;return be.nextSibling,r(U,()=>n.id),r(k,()=>n.mime||"Unknown type",Y),r(k,()=>_e(n.size),null),r(s,()=>n.sha256,A),r(s,()=>n.source_client_id||"Unknown",ue),r(s,()=>n.local_path||"None",be),r(s,()=>new Date(n.created_at).toLocaleString(),null),r(s,f(m,{get when(){return Object.keys(n.metadata).length>0},get children(){return[We(),"Metadata: ",Z(()=>JSON.stringify(n.metadata))]}}),null),v})()})}}),null),j(n=>{var v=h()!==u.Connected,y=h()!==u.Connected;return v!==n.e&&(X.disabled=n.e=v),y!==n.t&&(ie.disabled=n.t=y),n},{e:void 0,t:void 0}),e})()};pe("websocket-handler",{websocketUrl:"",autoConnect:!0,showDebugLog:!0},He);De(["click"]);
//# sourceMappingURL=websocket-components.js.map
