import{b as ne,a as x,f as P,t as d,i as s,c as m,g as q,h as te,m as G,S as $,d as me,F as fe}from"./index-iYDJ9E20.js";var he=d("<span>"),xe=d("<span class=user-count>(<!> user<!>)"),$e=d(`<div><style>
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
      </style><div>`),l=(a=>(a.Disconnected="disconnected",a.Connecting="connecting",a.Connected="connected",a.Error="error",a))(l||{});const oe=a=>{const[y,z]=x(Date.now()),v=()=>a.status??"disconnected",f=()=>a.showText??!0,C=()=>a.userCount??0,_=()=>a.showUserCount??!1,E=()=>a.compact??!1;P(()=>{const i=v();z(Date.now());const S=new CustomEvent("status-change",{detail:{status:i,timestamp:y()},bubbles:!0});setTimeout(()=>{const h=document.querySelector("websocket-status");h&&h.dispatchEvent(S)},0)});const T=()=>{switch(v()){case"disconnected":return"Offline";case"connecting":return"Connecting...";case"connected":return"Online";case"error":return"Connection Error";default:return"Unknown"}},L=()=>`status-indicator ${v()}`,k=()=>`status-text ${v()}`;return(()=>{var i=$e(),S=i.firstChild,h=S.nextSibling;return i.style.setProperty("display","inline-flex"),i.style.setProperty("align-items","center"),i.style.setProperty("gap","8px"),i.style.setProperty("font-family",'-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif'),i.style.setProperty("font-size","14px"),s(i,m($,{get when(){return G(()=>!!f())()&&!E()},get children(){var u=he();return s(u,T),q(()=>te(u,k())),u}}),null),s(i,m($,{get when(){return G(()=>!!(_()&&C()>0))()&&!E()},get children(){var u=xe(),O=u.firstChild,r=O.nextSibling,p=r.nextSibling,b=p.nextSibling;return b.nextSibling,s(u,C,r),s(u,()=>C()!==1?"s":"",b),u}}),null),q(()=>te(h,L())),i})()};ne("websocket-status",{status:"disconnected",showText:!0,userCount:0,showUserCount:!1,compact:!1},oe);var ye=d("<button>Disconnect"),ve=d("<div class=error-message>"),ke=d("<div class=debug-log>"),Se=d(`<div><style>
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
      </style><div class=container><div class=header><h2 class=title>WebSocket Handler</h2><div class=controls><button>Ping</button><button>Get Media Blobs</button></div></div><div class=status-section></div><div class=media-blobs><h3>Media Blobs (<!>)`),we=d("<button class=primary>Connect"),Ce=d('<div class=empty-state>No media blobs received yet. Click "Get Media Blobs" to fetch from server.'),_e=d("<br>"),Ee=d("<div class=media-blob><div class=media-blob-header><div class=media-blob-id></div><div class=media-blob-info> • </div></div><div class=media-blob-meta>SHA256: <br>Client: <br>Path: <br>Created: ");const Me=a=>{const y=()=>a.websocketUrl??"",z=()=>a.autoConnect??!0,v=()=>a.showDebugLog??!0,[f,C]=x(l.Disconnected),[_,E]=x(null),[T,L]=x([]),[k,i]=x([]),[S,h]=x(""),[u,O]=x(0);P(()=>{z()&&y()&&W()});const r=(e,...t)=>{const o=new Date().toLocaleTimeString(),c=t.length>0?`[${o}] ${e}: ${JSON.stringify(t,null,2)}`:`[${o}] ${e}`;L(g=>[...g.slice(-99),c]),console.log("[WebSocketHandler]",e,...t)},p=e=>{if(f()!==e){C(e),r(`Status changed to: ${e}`);const t=new CustomEvent("status-change",{detail:{status:e},bubbles:!0});setTimeout(()=>{const o=document.querySelector("websocket-handler");o&&o.dispatchEvent(t)},0)}},b=e=>{h(e),r(`Error: ${e}`)},R=()=>{h("")},W=()=>{if(!y()){b("WebSocket URL not provided");return}if(_()?.readyState===WebSocket.OPEN){r("Already connected");return}R(),p(l.Connecting),r(`Connecting to ${y()}`);try{const e=new WebSocket(y());E(e),re(e)}catch(e){b(`Connection failed: ${e}`),p(l.Error)}},N=()=>{r("Disconnecting...");const e=_();e&&(e.close(1e3,"Client disconnect"),E(null)),p(l.Disconnected)},re=e=>{e.onopen=()=>{r("Connected successfully"),p(l.Connected),R()},e.onclose=t=>{r("Connection closed",{code:t.code,reason:t.reason}),p(l.Disconnected)},e.onerror=t=>{r("Socket error",t),p(l.Error),b("Connection error occurred")},e.onmessage=t=>{se(t.data)}},se=e=>{r("Received raw message",e);try{const t=JSON.parse(e);switch(r("Parsed message",t),t.type){case"Welcome":r("Welcome received",t.data);break;case"Pong":r("Pong received");break;case"MediaBlobs":{r("Media blobs received",t.data);const o=t.data;i(o?.blobs||[]);const c=new CustomEvent("media-blobs-received",{detail:{blobs:k(),totalCount:o?.total_count},bubbles:!0});setTimeout(()=>{const g=document.querySelector("websocket-handler");g&&g.dispatchEvent(c)},0);break}case"MediaBlob":{r("Single media blob received",t.data);const o=t.data,c=new CustomEvent("media-blob-received",{detail:{blob:o?.blob},bubbles:!0});setTimeout(()=>{const g=document.querySelector("websocket-handler");g&&g.dispatchEvent(c)},0);break}case"Error":{r("Error message received",t.data);const o=t.data;b(o?.message||"Server error");break}case"ConnectionStatus":{r("Connection status update",t.data);const o=t.data;O(o?.user_count||0);break}default:r("Unknown message type",t)}}catch(t){r("Failed to parse message",{error:t instanceof Error?t.toString():String(t),rawMessage:e}),b(`Message parse error: ${t}`)}},D=e=>{const t=_();if(!t||t.readyState!==WebSocket.OPEN)return b("Cannot send message: not connected"),!1;try{const o=JSON.stringify(e);return t.send(o),r("Sent message",e),!0}catch(o){return b(`Send error: ${o}`),!1}},H=()=>D({type:"Ping"}),I=(e,t)=>D({type:"GetMediaBlobs",data:{limit:e,offset:t}}),ae=e=>D({type:"GetMediaBlob",data:{id:e}}),ie=e=>D({type:"UploadMediaBlob",data:{blob:e}}),ce=()=>{const e=document.querySelector("websocket-handler");e&&Object.assign(e,{ping:H,getMediaBlobs:I,getMediaBlob:ae,uploadMediaBlob:ie,connect:W,disconnect:N})};P(()=>{setTimeout(ce,0)});const le=e=>{if(!e)return"Unknown size";const t=["B","KB","MB","GB"];let o=e,c=0;for(;o>=1024&&c<t.length-1;)o/=1024,c++;return`${o.toFixed(1)} ${t[c]}`};return P(()=>N),(()=>{var e=Se(),t=e.firstChild,o=t.nextSibling,c=o.firstChild,g=c.firstChild,J=g.nextSibling,F=J.firstChild,A=F.nextSibling,K=c.nextSibling,U=K.nextSibling,Q=U.firstChild,de=Q.firstChild,V=de.nextSibling;return V.nextSibling,e.style.setProperty("display","block"),e.style.setProperty("font-family",'-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif'),F.$$click=H,A.$$click=()=>I(),s(J,m($,{get when(){return f()===l.Connected},get fallback(){return(()=>{var n=we();return n.$$click=W,n})()},get children(){var n=ye();return n.$$click=N,n}}),null),s(K,()=>oe({status:f(),userCount:u(),showUserCount:!0,showText:!0,compact:!1})),s(o,m($,{get when(){return S()},get children(){var n=ve();return s(n,S),n}}),U),s(o,m($,{get when(){return v()},get children(){var n=ke();return s(n,()=>T().join(`
`)),n}}),U),s(Q,()=>k().length,V),s(U,m($,{get when(){return k().length>0},get fallback(){return Ce()},get children(){return m(fe,{get each(){return k()},children:n=>(()=>{var M=Ee(),B=M.firstChild,X=B.firstChild,j=X.nextSibling,ue=j.firstChild,w=B.nextSibling,be=w.firstChild,Y=be.nextSibling,ge=Y.nextSibling,Z=ge.nextSibling,pe=Z.nextSibling,ee=pe.nextSibling;return ee.nextSibling,s(X,()=>n.id),s(j,()=>n.mime||"Unknown type",ue),s(j,()=>le(n.size),null),s(w,()=>n.sha256,Y),s(w,()=>n.source_client_id||"Unknown",Z),s(w,()=>n.local_path||"None",ee),s(w,()=>new Date(n.created_at).toLocaleString(),null),s(w,m($,{get when(){return Object.keys(n.metadata).length>0},get children(){return[_e(),"Metadata: ",G(()=>JSON.stringify(n.metadata))]}}),null),M})()})}}),null),q(n=>{var M=f()!==l.Connected,B=f()!==l.Connected;return M!==n.e&&(F.disabled=n.e=M),B!==n.t&&(A.disabled=n.t=B),n},{e:void 0,t:void 0}),e})()};ne("websocket-handler",{websocketUrl:"",autoConnect:!0,showDebugLog:!0},Me);me(["click"]);
//# sourceMappingURL=websocket-components.js.map
