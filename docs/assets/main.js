/*! For license information please see main.js.LICENSE.txt */
"use strict";(()=>{var e,t,r=Object.create,n=Object.defineProperty,i=Object.getOwnPropertyDescriptor,s=Object.getOwnPropertyNames,o=Object.getPrototypeOf,a=Object.prototype.hasOwnProperty,l=(e=(e,t)=>{!function(){var r,n=function(e){var t=new n.Builder;return t.pipeline.add(n.trimmer,n.stopWordFilter,n.stemmer),t.searchPipeline.add(n.stemmer),e.call(t,t),t.build()};n.version="2.3.9",n.utils={},n.utils.warn=(r=this,function(e){r.console&&console.warn&&console.warn(e)}),n.utils.asString=function(e){return null==e?"":e.toString()},n.utils.clone=function(e){if(null==e)return e;for(var t=Object.create(null),r=Object.keys(e),n=0;n<r.length;n++){var i=r[n],s=e[i];if(Array.isArray(s))t[i]=s.slice();else{if("string"!=typeof s&&"number"!=typeof s&&"boolean"!=typeof s)throw new TypeError("clone is not deep and does not support nested objects");t[i]=s}}return t},n.FieldRef=function(e,t,r){this.docRef=e,this.fieldName=t,this._stringValue=r},n.FieldRef.joiner="/",n.FieldRef.fromString=function(e){var t=e.indexOf(n.FieldRef.joiner);if(-1===t)throw"malformed field ref string";var r=e.slice(0,t),i=e.slice(t+1);return new n.FieldRef(i,r,e)},n.FieldRef.prototype.toString=function(){return null==this._stringValue&&(this._stringValue=this.fieldName+n.FieldRef.joiner+this.docRef),this._stringValue},n.Set=function(e){if(this.elements=Object.create(null),e){this.length=e.length;for(var t=0;t<this.length;t++)this.elements[e[t]]=!0}else this.length=0},n.Set.complete={intersect:function(e){return e},union:function(){return this},contains:function(){return!0}},n.Set.empty={intersect:function(){return this},union:function(e){return e},contains:function(){return!1}},n.Set.prototype.contains=function(e){return!!this.elements[e]},n.Set.prototype.intersect=function(e){var t,r,i,s=[];if(e===n.Set.complete)return this;if(e===n.Set.empty)return e;this.length<e.length?(t=this,r=e):(t=e,r=this),i=Object.keys(t.elements);for(var o=0;o<i.length;o++){var a=i[o];a in r.elements&&s.push(a)}return new n.Set(s)},n.Set.prototype.union=function(e){return e===n.Set.complete?n.Set.complete:e===n.Set.empty?this:new n.Set(Object.keys(this.elements).concat(Object.keys(e.elements)))},n.idf=function(e,t){var r=0;for(var n in e)"_index"!=n&&(r+=Object.keys(e[n]).length);var i=(t-r+.5)/(r+.5);return Math.log(1+Math.abs(i))},n.Token=function(e,t){this.str=e||"",this.metadata=t||{}},n.Token.prototype.toString=function(){return this.str},n.Token.prototype.update=function(e){return this.str=e(this.str,this.metadata),this},n.Token.prototype.clone=function(e){return e=e||function(e){return e},new n.Token(e(this.str,this.metadata),this.metadata)},n.tokenizer=function(e,t){if(null==e||null==e)return[];if(Array.isArray(e))return e.map((function(e){return new n.Token(n.utils.asString(e).toLowerCase(),n.utils.clone(t))}));for(var r=e.toString().toLowerCase(),i=r.length,s=[],o=0,a=0;o<=i;o++){var l=o-a;if(r.charAt(o).match(n.tokenizer.separator)||o==i){if(l>0){var u=n.utils.clone(t)||{};u.position=[a,l],u.index=s.length,s.push(new n.Token(r.slice(a,o),u))}a=o+1}}return s},n.tokenizer.separator=/[\s\-]+/,n.Pipeline=function(){this._stack=[]},n.Pipeline.registeredFunctions=Object.create(null),n.Pipeline.registerFunction=function(e,t){t in this.registeredFunctions&&n.utils.warn("Overwriting existing registered function: "+t),e.label=t,n.Pipeline.registeredFunctions[e.label]=e},n.Pipeline.warnIfFunctionNotRegistered=function(e){e.label&&e.label in this.registeredFunctions||n.utils.warn("Function is not registered with pipeline. This may cause problems when serialising the index.\n",e)},n.Pipeline.load=function(e){var t=new n.Pipeline;return e.forEach((function(e){var r=n.Pipeline.registeredFunctions[e];if(!r)throw new Error("Cannot load unregistered function: "+e);t.add(r)})),t},n.Pipeline.prototype.add=function(){Array.prototype.slice.call(arguments).forEach((function(e){n.Pipeline.warnIfFunctionNotRegistered(e),this._stack.push(e)}),this)},n.Pipeline.prototype.after=function(e,t){n.Pipeline.warnIfFunctionNotRegistered(t);var r=this._stack.indexOf(e);if(-1==r)throw new Error("Cannot find existingFn");r+=1,this._stack.splice(r,0,t)},n.Pipeline.prototype.before=function(e,t){n.Pipeline.warnIfFunctionNotRegistered(t);var r=this._stack.indexOf(e);if(-1==r)throw new Error("Cannot find existingFn");this._stack.splice(r,0,t)},n.Pipeline.prototype.remove=function(e){var t=this._stack.indexOf(e);-1!=t&&this._stack.splice(t,1)},n.Pipeline.prototype.run=function(e){for(var t=this._stack.length,r=0;r<t;r++){for(var n=this._stack[r],i=[],s=0;s<e.length;s++){var o=n(e[s],s,e);if(null!=o&&""!==o)if(Array.isArray(o))for(var a=0;a<o.length;a++)i.push(o[a]);else i.push(o)}e=i}return e},n.Pipeline.prototype.runString=function(e,t){var r=new n.Token(e,t);return this.run([r]).map((function(e){return e.toString()}))},n.Pipeline.prototype.reset=function(){this._stack=[]},n.Pipeline.prototype.toJSON=function(){return this._stack.map((function(e){return n.Pipeline.warnIfFunctionNotRegistered(e),e.label}))},n.Vector=function(e){this._magnitude=0,this.elements=e||[]},n.Vector.prototype.positionForIndex=function(e){if(0==this.elements.length)return 0;for(var t=0,r=this.elements.length/2,n=r-t,i=Math.floor(n/2),s=this.elements[2*i];n>1&&(s<e&&(t=i),s>e&&(r=i),s!=e);)n=r-t,i=t+Math.floor(n/2),s=this.elements[2*i];return s==e||s>e?2*i:s<e?2*(i+1):void 0},n.Vector.prototype.insert=function(e,t){this.upsert(e,t,(function(){throw"duplicate index"}))},n.Vector.prototype.upsert=function(e,t,r){this._magnitude=0;var n=this.positionForIndex(e);this.elements[n]==e?this.elements[n+1]=r(this.elements[n+1],t):this.elements.splice(n,0,e,t)},n.Vector.prototype.magnitude=function(){if(this._magnitude)return this._magnitude;for(var e=0,t=this.elements.length,r=1;r<t;r+=2){var n=this.elements[r];e+=n*n}return this._magnitude=Math.sqrt(e)},n.Vector.prototype.dot=function(e){for(var t=0,r=this.elements,n=e.elements,i=r.length,s=n.length,o=0,a=0,l=0,u=0;l<i&&u<s;)(o=r[l])<(a=n[u])?l+=2:o>a?u+=2:o==a&&(t+=r[l+1]*n[u+1],l+=2,u+=2);return t},n.Vector.prototype.similarity=function(e){return this.dot(e)/this.magnitude()||0},n.Vector.prototype.toArray=function(){for(var e=new Array(this.elements.length/2),t=1,r=0;t<this.elements.length;t+=2,r++)e[r]=this.elements[t];return e},n.Vector.prototype.toJSON=function(){return this.elements},n.stemmer=function(){var e={ational:"ate",tional:"tion",enci:"ence",anci:"ance",izer:"ize",bli:"ble",alli:"al",entli:"ent",eli:"e",ousli:"ous",ization:"ize",ation:"ate",ator:"ate",alism:"al",iveness:"ive",fulness:"ful",ousness:"ous",aliti:"al",iviti:"ive",biliti:"ble",logi:"log"},t={icate:"ic",ative:"",alize:"al",iciti:"ic",ical:"ic",ful:"",ness:""},r="[aeiouy]",n="[^aeiou][^aeiouy]*",i=r+"[aeiou]*",s="^("+n+")?"+i+n+"("+i+")?$",o="^("+n+")?"+i+n+i+n,a="^("+n+")?"+r,l=new RegExp("^("+n+")?"+i+n),u=new RegExp(o),c=new RegExp(s),d=new RegExp(a),h=/^(.+?)(ss|i)es$/,f=/^(.+?)([^s])s$/,p=/^(.+?)eed$/,m=/^(.+?)(ed|ing)$/,y=/.$/,g=/(at|bl|iz)$/,v=new RegExp("([^aeiouylsz])\\1$"),x=new RegExp("^"+n+r+"[^aeiouwxy]$"),w=/^(.+?[^aeiou])y$/,E=/^(.+?)(ational|tional|enci|anci|izer|bli|alli|entli|eli|ousli|ization|ation|ator|alism|iveness|fulness|ousness|aliti|iviti|biliti|logi)$/,L=/^(.+?)(icate|ative|alize|iciti|ical|ful|ness)$/,b=/^(.+?)(al|ance|ence|er|ic|able|ible|ant|ement|ment|ent|ou|ism|ate|iti|ous|ive|ize)$/,k=/^(.+?)(s|t)(ion)$/,S=/^(.+?)e$/,Q=/ll$/,P=new RegExp("^"+n+r+"[^aeiouwxy]$"),T=function(r){var n,i,s,o,a,T,O;if(r.length<3)return r;if("y"==(s=r.substr(0,1))&&(r=s.toUpperCase()+r.substr(1)),a=f,(o=h).test(r)?r=r.replace(o,"$1$2"):a.test(r)&&(r=r.replace(a,"$1$2")),a=m,(o=p).test(r)){var I=o.exec(r);(o=l).test(I[1])&&(o=y,r=r.replace(o,""))}else a.test(r)&&(n=(I=a.exec(r))[1],(a=d).test(n)&&(T=v,O=x,(a=g).test(r=n)?r+="e":T.test(r)?(o=y,r=r.replace(o,"")):O.test(r)&&(r+="e")));return(o=w).test(r)&&(r=(n=(I=o.exec(r))[1])+"i"),(o=E).test(r)&&(n=(I=o.exec(r))[1],i=I[2],(o=l).test(n)&&(r=n+e[i])),(o=L).test(r)&&(n=(I=o.exec(r))[1],i=I[2],(o=l).test(n)&&(r=n+t[i])),a=k,(o=b).test(r)?(n=(I=o.exec(r))[1],(o=u).test(n)&&(r=n)):a.test(r)&&(n=(I=a.exec(r))[1]+I[2],(a=u).test(n)&&(r=n)),(o=S).test(r)&&(n=(I=o.exec(r))[1],a=c,T=P,((o=u).test(n)||a.test(n)&&!T.test(n))&&(r=n)),a=u,(o=Q).test(r)&&a.test(r)&&(o=y,r=r.replace(o,"")),"y"==s&&(r=s.toLowerCase()+r.substr(1)),r};return function(e){return e.update(T)}}(),n.Pipeline.registerFunction(n.stemmer,"stemmer"),n.generateStopWordFilter=function(e){var t=e.reduce((function(e,t){return e[t]=t,e}),{});return function(e){if(e&&t[e.toString()]!==e.toString())return e}},n.stopWordFilter=n.generateStopWordFilter(["a","able","about","across","after","all","almost","also","am","among","an","and","any","are","as","at","be","because","been","but","by","can","cannot","could","dear","did","do","does","either","else","ever","every","for","from","get","got","had","has","have","he","her","hers","him","his","how","however","i","if","in","into","is","it","its","just","least","let","like","likely","may","me","might","most","must","my","neither","no","nor","not","of","off","often","on","only","or","other","our","own","rather","said","say","says","she","should","since","so","some","than","that","the","their","them","then","there","these","they","this","tis","to","too","twas","us","wants","was","we","were","what","when","where","which","while","who","whom","why","will","with","would","yet","you","your"]),n.Pipeline.registerFunction(n.stopWordFilter,"stopWordFilter"),n.trimmer=function(e){return e.update((function(e){return e.replace(/^\W+/,"").replace(/\W+$/,"")}))},n.Pipeline.registerFunction(n.trimmer,"trimmer"),n.TokenSet=function(){this.final=!1,this.edges={},this.id=n.TokenSet._nextId,n.TokenSet._nextId+=1},n.TokenSet._nextId=1,n.TokenSet.fromArray=function(e){for(var t=new n.TokenSet.Builder,r=0,i=e.length;r<i;r++)t.insert(e[r]);return t.finish(),t.root},n.TokenSet.fromClause=function(e){return"editDistance"in e?n.TokenSet.fromFuzzyString(e.term,e.editDistance):n.TokenSet.fromString(e.term)},n.TokenSet.fromFuzzyString=function(e,t){for(var r=new n.TokenSet,i=[{node:r,editsRemaining:t,str:e}];i.length;){var s=i.pop();if(s.str.length>0){var o,a=s.str.charAt(0);a in s.node.edges?o=s.node.edges[a]:(o=new n.TokenSet,s.node.edges[a]=o),1==s.str.length&&(o.final=!0),i.push({node:o,editsRemaining:s.editsRemaining,str:s.str.slice(1)})}if(0!=s.editsRemaining){if("*"in s.node.edges)var l=s.node.edges["*"];else l=new n.TokenSet,s.node.edges["*"]=l;if(0==s.str.length&&(l.final=!0),i.push({node:l,editsRemaining:s.editsRemaining-1,str:s.str}),s.str.length>1&&i.push({node:s.node,editsRemaining:s.editsRemaining-1,str:s.str.slice(1)}),1==s.str.length&&(s.node.final=!0),s.str.length>=1){if("*"in s.node.edges)var u=s.node.edges["*"];else u=new n.TokenSet,s.node.edges["*"]=u;1==s.str.length&&(u.final=!0),i.push({node:u,editsRemaining:s.editsRemaining-1,str:s.str.slice(1)})}if(s.str.length>1){var c,d=s.str.charAt(0),h=s.str.charAt(1);h in s.node.edges?c=s.node.edges[h]:(c=new n.TokenSet,s.node.edges[h]=c),1==s.str.length&&(c.final=!0),i.push({node:c,editsRemaining:s.editsRemaining-1,str:d+s.str.slice(2)})}}}return r},n.TokenSet.fromString=function(e){for(var t=new n.TokenSet,r=t,i=0,s=e.length;i<s;i++){var o=e[i],a=i==s-1;if("*"==o)t.edges[o]=t,t.final=a;else{var l=new n.TokenSet;l.final=a,t.edges[o]=l,t=l}}return r},n.TokenSet.prototype.toArray=function(){for(var e=[],t=[{prefix:"",node:this}];t.length;){var r=t.pop(),n=Object.keys(r.node.edges),i=n.length;r.node.final&&(r.prefix.charAt(0),e.push(r.prefix));for(var s=0;s<i;s++){var o=n[s];t.push({prefix:r.prefix.concat(o),node:r.node.edges[o]})}}return e},n.TokenSet.prototype.toString=function(){if(this._str)return this._str;for(var e=this.final?"1":"0",t=Object.keys(this.edges).sort(),r=t.length,n=0;n<r;n++){var i=t[n];e=e+i+this.edges[i].id}return e},n.TokenSet.prototype.intersect=function(e){for(var t=new n.TokenSet,r=void 0,i=[{qNode:e,output:t,node:this}];i.length;){r=i.pop();for(var s=Object.keys(r.qNode.edges),o=s.length,a=Object.keys(r.node.edges),l=a.length,u=0;u<o;u++)for(var c=s[u],d=0;d<l;d++){var h=a[d];if(h==c||"*"==c){var f=r.node.edges[h],p=r.qNode.edges[c],m=f.final&&p.final,y=void 0;h in r.output.edges?(y=r.output.edges[h]).final=y.final||m:((y=new n.TokenSet).final=m,r.output.edges[h]=y),i.push({qNode:p,output:y,node:f})}}}return t},n.TokenSet.Builder=function(){this.previousWord="",this.root=new n.TokenSet,this.uncheckedNodes=[],this.minimizedNodes={}},n.TokenSet.Builder.prototype.insert=function(e){var t,r=0;if(e<this.previousWord)throw new Error("Out of order word insertion");for(var i=0;i<e.length&&i<this.previousWord.length&&e[i]==this.previousWord[i];i++)r++;for(this.minimize(r),t=0==this.uncheckedNodes.length?this.root:this.uncheckedNodes[this.uncheckedNodes.length-1].child,i=r;i<e.length;i++){var s=new n.TokenSet,o=e[i];t.edges[o]=s,this.uncheckedNodes.push({parent:t,char:o,child:s}),t=s}t.final=!0,this.previousWord=e},n.TokenSet.Builder.prototype.finish=function(){this.minimize(0)},n.TokenSet.Builder.prototype.minimize=function(e){for(var t=this.uncheckedNodes.length-1;t>=e;t--){var r=this.uncheckedNodes[t],n=r.child.toString();n in this.minimizedNodes?r.parent.edges[r.char]=this.minimizedNodes[n]:(r.child._str=n,this.minimizedNodes[n]=r.child),this.uncheckedNodes.pop()}},n.Index=function(e){this.invertedIndex=e.invertedIndex,this.fieldVectors=e.fieldVectors,this.tokenSet=e.tokenSet,this.fields=e.fields,this.pipeline=e.pipeline},n.Index.prototype.search=function(e){return this.query((function(t){new n.QueryParser(e,t).parse()}))},n.Index.prototype.query=function(e){for(var t=new n.Query(this.fields),r=Object.create(null),i=Object.create(null),s=Object.create(null),o=Object.create(null),a=Object.create(null),l=0;l<this.fields.length;l++)i[this.fields[l]]=new n.Vector;for(e.call(t,t),l=0;l<t.clauses.length;l++){var u,c=t.clauses[l],d=n.Set.empty;u=c.usePipeline?this.pipeline.runString(c.term,{fields:c.fields}):[c.term];for(var h=0;h<u.length;h++){var f=u[h];c.term=f;var p=n.TokenSet.fromClause(c),m=this.tokenSet.intersect(p).toArray();if(0===m.length&&c.presence===n.Query.presence.REQUIRED){for(var y=0;y<c.fields.length;y++)o[R=c.fields[y]]=n.Set.empty;break}for(var g=0;g<m.length;g++){var v=m[g],x=this.invertedIndex[v],w=x._index;for(y=0;y<c.fields.length;y++){var E=x[R=c.fields[y]],L=Object.keys(E),b=v+"/"+R,k=new n.Set(L);if(c.presence==n.Query.presence.REQUIRED&&(d=d.union(k),void 0===o[R]&&(o[R]=n.Set.complete)),c.presence!=n.Query.presence.PROHIBITED){if(i[R].upsert(w,c.boost,(function(e,t){return e+t})),!s[b]){for(var S=0;S<L.length;S++){var Q,P=L[S],T=new n.FieldRef(P,R),O=E[P];void 0===(Q=r[T])?r[T]=new n.MatchData(v,R,O):Q.add(v,R,O)}s[b]=!0}}else void 0===a[R]&&(a[R]=n.Set.empty),a[R]=a[R].union(k)}}}if(c.presence===n.Query.presence.REQUIRED)for(y=0;y<c.fields.length;y++)o[R=c.fields[y]]=o[R].intersect(d)}var I=n.Set.complete,C=n.Set.empty;for(l=0;l<this.fields.length;l++){var R;o[R=this.fields[l]]&&(I=I.intersect(o[R])),a[R]&&(C=C.union(a[R]))}var F=Object.keys(r),D=[],N=Object.create(null);if(t.isNegated())for(F=Object.keys(this.fieldVectors),l=0;l<F.length;l++){T=F[l];var j=n.FieldRef.fromString(T);r[T]=new n.MatchData}for(l=0;l<F.length;l++){var _=(j=n.FieldRef.fromString(F[l])).docRef;if(I.contains(_)&&!C.contains(_)){var A,V=this.fieldVectors[j],B=i[j.fieldName].similarity(V);if(void 0!==(A=N[_]))A.score+=B,A.matchData.combine(r[j]);else{var M={ref:_,score:B,matchData:r[j]};N[_]=M,D.push(M)}}}return D.sort((function(e,t){return t.score-e.score}))},n.Index.prototype.toJSON=function(){var e=Object.keys(this.invertedIndex).sort().map((function(e){return[e,this.invertedIndex[e]]}),this),t=Object.keys(this.fieldVectors).map((function(e){return[e,this.fieldVectors[e].toJSON()]}),this);return{version:n.version,fields:this.fields,fieldVectors:t,invertedIndex:e,pipeline:this.pipeline.toJSON()}},n.Index.load=function(e){var t={},r={},i=e.fieldVectors,s=Object.create(null),o=e.invertedIndex,a=new n.TokenSet.Builder,l=n.Pipeline.load(e.pipeline);e.version!=n.version&&n.utils.warn("Version mismatch when loading serialised index. Current version of lunr '"+n.version+"' does not match serialized index '"+e.version+"'");for(var u=0;u<i.length;u++){var c=(h=i[u])[0],d=h[1];r[c]=new n.Vector(d)}for(u=0;u<o.length;u++){var h,f=(h=o[u])[0],p=h[1];a.insert(f),s[f]=p}return a.finish(),t.fields=e.fields,t.fieldVectors=r,t.invertedIndex=s,t.tokenSet=a.root,t.pipeline=l,new n.Index(t)},n.Builder=function(){this._ref="id",this._fields=Object.create(null),this._documents=Object.create(null),this.invertedIndex=Object.create(null),this.fieldTermFrequencies={},this.fieldLengths={},this.tokenizer=n.tokenizer,this.pipeline=new n.Pipeline,this.searchPipeline=new n.Pipeline,this.documentCount=0,this._b=.75,this._k1=1.2,this.termIndex=0,this.metadataWhitelist=[]},n.Builder.prototype.ref=function(e){this._ref=e},n.Builder.prototype.field=function(e,t){if(/\//.test(e))throw new RangeError("Field '"+e+"' contains illegal character '/'");this._fields[e]=t||{}},n.Builder.prototype.b=function(e){this._b=e<0?0:e>1?1:e},n.Builder.prototype.k1=function(e){this._k1=e},n.Builder.prototype.add=function(e,t){var r=e[this._ref],i=Object.keys(this._fields);this._documents[r]=t||{},this.documentCount+=1;for(var s=0;s<i.length;s++){var o=i[s],a=this._fields[o].extractor,l=a?a(e):e[o],u=this.tokenizer(l,{fields:[o]}),c=this.pipeline.run(u),d=new n.FieldRef(r,o),h=Object.create(null);this.fieldTermFrequencies[d]=h,this.fieldLengths[d]=0,this.fieldLengths[d]+=c.length;for(var f=0;f<c.length;f++){var p=c[f];if(null==h[p]&&(h[p]=0),h[p]+=1,null==this.invertedIndex[p]){var m=Object.create(null);m._index=this.termIndex,this.termIndex+=1;for(var y=0;y<i.length;y++)m[i[y]]=Object.create(null);this.invertedIndex[p]=m}null==this.invertedIndex[p][o][r]&&(this.invertedIndex[p][o][r]=Object.create(null));for(var g=0;g<this.metadataWhitelist.length;g++){var v=this.metadataWhitelist[g],x=p.metadata[v];null==this.invertedIndex[p][o][r][v]&&(this.invertedIndex[p][o][r][v]=[]),this.invertedIndex[p][o][r][v].push(x)}}}},n.Builder.prototype.calculateAverageFieldLengths=function(){for(var e=Object.keys(this.fieldLengths),t=e.length,r={},i={},s=0;s<t;s++){var o=n.FieldRef.fromString(e[s]),a=o.fieldName;i[a]||(i[a]=0),i[a]+=1,r[a]||(r[a]=0),r[a]+=this.fieldLengths[o]}var l=Object.keys(this._fields);for(s=0;s<l.length;s++){var u=l[s];r[u]=r[u]/i[u]}this.averageFieldLength=r},n.Builder.prototype.createFieldVectors=function(){for(var e={},t=Object.keys(this.fieldTermFrequencies),r=t.length,i=Object.create(null),s=0;s<r;s++){for(var o=n.FieldRef.fromString(t[s]),a=o.fieldName,l=this.fieldLengths[o],u=new n.Vector,c=this.fieldTermFrequencies[o],d=Object.keys(c),h=d.length,f=this._fields[a].boost||1,p=this._documents[o.docRef].boost||1,m=0;m<h;m++){var y,g,v,x=d[m],w=c[x],E=this.invertedIndex[x]._index;void 0===i[x]?(y=n.idf(this.invertedIndex[x],this.documentCount),i[x]=y):y=i[x],g=y*((this._k1+1)*w)/(this._k1*(1-this._b+this._b*(l/this.averageFieldLength[a]))+w),g*=f,g*=p,v=Math.round(1e3*g)/1e3,u.insert(E,v)}e[o]=u}this.fieldVectors=e},n.Builder.prototype.createTokenSet=function(){this.tokenSet=n.TokenSet.fromArray(Object.keys(this.invertedIndex).sort())},n.Builder.prototype.build=function(){return this.calculateAverageFieldLengths(),this.createFieldVectors(),this.createTokenSet(),new n.Index({invertedIndex:this.invertedIndex,fieldVectors:this.fieldVectors,tokenSet:this.tokenSet,fields:Object.keys(this._fields),pipeline:this.searchPipeline})},n.Builder.prototype.use=function(e){var t=Array.prototype.slice.call(arguments,1);t.unshift(this),e.apply(this,t)},n.MatchData=function(e,t,r){for(var n=Object.create(null),i=Object.keys(r||{}),s=0;s<i.length;s++){var o=i[s];n[o]=r[o].slice()}this.metadata=Object.create(null),void 0!==e&&(this.metadata[e]=Object.create(null),this.metadata[e][t]=n)},n.MatchData.prototype.combine=function(e){for(var t=Object.keys(e.metadata),r=0;r<t.length;r++){var n=t[r],i=Object.keys(e.metadata[n]);null==this.metadata[n]&&(this.metadata[n]=Object.create(null));for(var s=0;s<i.length;s++){var o=i[s],a=Object.keys(e.metadata[n][o]);null==this.metadata[n][o]&&(this.metadata[n][o]=Object.create(null));for(var l=0;l<a.length;l++){var u=a[l];null==this.metadata[n][o][u]?this.metadata[n][o][u]=e.metadata[n][o][u]:this.metadata[n][o][u]=this.metadata[n][o][u].concat(e.metadata[n][o][u])}}}},n.MatchData.prototype.add=function(e,t,r){if(!(e in this.metadata))return this.metadata[e]=Object.create(null),void(this.metadata[e][t]=r);if(t in this.metadata[e])for(var n=Object.keys(r),i=0;i<n.length;i++){var s=n[i];s in this.metadata[e][t]?this.metadata[e][t][s]=this.metadata[e][t][s].concat(r[s]):this.metadata[e][t][s]=r[s]}else this.metadata[e][t]=r},n.Query=function(e){this.clauses=[],this.allFields=e},n.Query.wildcard=new String("*"),n.Query.wildcard.NONE=0,n.Query.wildcard.LEADING=1,n.Query.wildcard.TRAILING=2,n.Query.presence={OPTIONAL:1,REQUIRED:2,PROHIBITED:3},n.Query.prototype.clause=function(e){return"fields"in e||(e.fields=this.allFields),"boost"in e||(e.boost=1),"usePipeline"in e||(e.usePipeline=!0),"wildcard"in e||(e.wildcard=n.Query.wildcard.NONE),e.wildcard&n.Query.wildcard.LEADING&&e.term.charAt(0)!=n.Query.wildcard&&(e.term="*"+e.term),e.wildcard&n.Query.wildcard.TRAILING&&e.term.slice(-1)!=n.Query.wildcard&&(e.term=e.term+"*"),"presence"in e||(e.presence=n.Query.presence.OPTIONAL),this.clauses.push(e),this},n.Query.prototype.isNegated=function(){for(var e=0;e<this.clauses.length;e++)if(this.clauses[e].presence!=n.Query.presence.PROHIBITED)return!1;return!0},n.Query.prototype.term=function(e,t){if(Array.isArray(e))return e.forEach((function(e){this.term(e,n.utils.clone(t))}),this),this;var r=t||{};return r.term=e.toString(),this.clause(r),this},n.QueryParseError=function(e,t,r){this.name="QueryParseError",this.message=e,this.start=t,this.end=r},n.QueryParseError.prototype=new Error,n.QueryLexer=function(e){this.lexemes=[],this.str=e,this.length=e.length,this.pos=0,this.start=0,this.escapeCharPositions=[]},n.QueryLexer.prototype.run=function(){for(var e=n.QueryLexer.lexText;e;)e=e(this)},n.QueryLexer.prototype.sliceString=function(){for(var e=[],t=this.start,r=this.pos,n=0;n<this.escapeCharPositions.length;n++)r=this.escapeCharPositions[n],e.push(this.str.slice(t,r)),t=r+1;return e.push(this.str.slice(t,this.pos)),this.escapeCharPositions.length=0,e.join("")},n.QueryLexer.prototype.emit=function(e){this.lexemes.push({type:e,str:this.sliceString(),start:this.start,end:this.pos}),this.start=this.pos},n.QueryLexer.prototype.escapeCharacter=function(){this.escapeCharPositions.push(this.pos-1),this.pos+=1},n.QueryLexer.prototype.next=function(){if(this.pos>=this.length)return n.QueryLexer.EOS;var e=this.str.charAt(this.pos);return this.pos+=1,e},n.QueryLexer.prototype.width=function(){return this.pos-this.start},n.QueryLexer.prototype.ignore=function(){this.start==this.pos&&(this.pos+=1),this.start=this.pos},n.QueryLexer.prototype.backup=function(){this.pos-=1},n.QueryLexer.prototype.acceptDigitRun=function(){var e,t;do{t=(e=this.next()).charCodeAt(0)}while(t>47&&t<58);e!=n.QueryLexer.EOS&&this.backup()},n.QueryLexer.prototype.more=function(){return this.pos<this.length},n.QueryLexer.EOS="EOS",n.QueryLexer.FIELD="FIELD",n.QueryLexer.TERM="TERM",n.QueryLexer.EDIT_DISTANCE="EDIT_DISTANCE",n.QueryLexer.BOOST="BOOST",n.QueryLexer.PRESENCE="PRESENCE",n.QueryLexer.lexField=function(e){return e.backup(),e.emit(n.QueryLexer.FIELD),e.ignore(),n.QueryLexer.lexText},n.QueryLexer.lexTerm=function(e){if(e.width()>1&&(e.backup(),e.emit(n.QueryLexer.TERM)),e.ignore(),e.more())return n.QueryLexer.lexText},n.QueryLexer.lexEditDistance=function(e){return e.ignore(),e.acceptDigitRun(),e.emit(n.QueryLexer.EDIT_DISTANCE),n.QueryLexer.lexText},n.QueryLexer.lexBoost=function(e){return e.ignore(),e.acceptDigitRun(),e.emit(n.QueryLexer.BOOST),n.QueryLexer.lexText},n.QueryLexer.lexEOS=function(e){e.width()>0&&e.emit(n.QueryLexer.TERM)},n.QueryLexer.termSeparator=n.tokenizer.separator,n.QueryLexer.lexText=function(e){for(;;){var t=e.next();if(t==n.QueryLexer.EOS)return n.QueryLexer.lexEOS;if(92!=t.charCodeAt(0)){if(":"==t)return n.QueryLexer.lexField;if("~"==t)return e.backup(),e.width()>0&&e.emit(n.QueryLexer.TERM),n.QueryLexer.lexEditDistance;if("^"==t)return e.backup(),e.width()>0&&e.emit(n.QueryLexer.TERM),n.QueryLexer.lexBoost;if("+"==t&&1===e.width()||"-"==t&&1===e.width())return e.emit(n.QueryLexer.PRESENCE),n.QueryLexer.lexText;if(t.match(n.QueryLexer.termSeparator))return n.QueryLexer.lexTerm}else e.escapeCharacter()}},n.QueryParser=function(e,t){this.lexer=new n.QueryLexer(e),this.query=t,this.currentClause={},this.lexemeIdx=0},n.QueryParser.prototype.parse=function(){this.lexer.run(),this.lexemes=this.lexer.lexemes;for(var e=n.QueryParser.parseClause;e;)e=e(this);return this.query},n.QueryParser.prototype.peekLexeme=function(){return this.lexemes[this.lexemeIdx]},n.QueryParser.prototype.consumeLexeme=function(){var e=this.peekLexeme();return this.lexemeIdx+=1,e},n.QueryParser.prototype.nextClause=function(){var e=this.currentClause;this.query.clause(e),this.currentClause={}},n.QueryParser.parseClause=function(e){var t=e.peekLexeme();if(null!=t)switch(t.type){case n.QueryLexer.PRESENCE:return n.QueryParser.parsePresence;case n.QueryLexer.FIELD:return n.QueryParser.parseField;case n.QueryLexer.TERM:return n.QueryParser.parseTerm;default:var r="expected either a field or a term, found "+t.type;throw t.str.length>=1&&(r+=" with value '"+t.str+"'"),new n.QueryParseError(r,t.start,t.end)}},n.QueryParser.parsePresence=function(e){var t=e.consumeLexeme();if(null!=t){switch(t.str){case"-":e.currentClause.presence=n.Query.presence.PROHIBITED;break;case"+":e.currentClause.presence=n.Query.presence.REQUIRED;break;default:var r="unrecognised presence operator'"+t.str+"'";throw new n.QueryParseError(r,t.start,t.end)}var i=e.peekLexeme();if(null==i)throw r="expecting term or field, found nothing",new n.QueryParseError(r,t.start,t.end);switch(i.type){case n.QueryLexer.FIELD:return n.QueryParser.parseField;case n.QueryLexer.TERM:return n.QueryParser.parseTerm;default:throw r="expecting term or field, found '"+i.type+"'",new n.QueryParseError(r,i.start,i.end)}}},n.QueryParser.parseField=function(e){var t=e.consumeLexeme();if(null!=t){if(-1==e.query.allFields.indexOf(t.str)){var r=e.query.allFields.map((function(e){return"'"+e+"'"})).join(", "),i="unrecognised field '"+t.str+"', possible fields: "+r;throw new n.QueryParseError(i,t.start,t.end)}e.currentClause.fields=[t.str];var s=e.peekLexeme();if(null==s)throw i="expecting term, found nothing",new n.QueryParseError(i,t.start,t.end);if(s.type===n.QueryLexer.TERM)return n.QueryParser.parseTerm;throw i="expecting term, found '"+s.type+"'",new n.QueryParseError(i,s.start,s.end)}},n.QueryParser.parseTerm=function(e){var t=e.consumeLexeme();if(null!=t){e.currentClause.term=t.str.toLowerCase(),-1!=t.str.indexOf("*")&&(e.currentClause.usePipeline=!1);var r=e.peekLexeme();if(null==r)return void e.nextClause();switch(r.type){case n.QueryLexer.TERM:return e.nextClause(),n.QueryParser.parseTerm;case n.QueryLexer.FIELD:return e.nextClause(),n.QueryParser.parseField;case n.QueryLexer.EDIT_DISTANCE:return n.QueryParser.parseEditDistance;case n.QueryLexer.BOOST:return n.QueryParser.parseBoost;case n.QueryLexer.PRESENCE:return e.nextClause(),n.QueryParser.parsePresence;default:var i="Unexpected lexeme type '"+r.type+"'";throw new n.QueryParseError(i,r.start,r.end)}}},n.QueryParser.parseEditDistance=function(e){var t=e.consumeLexeme();if(null!=t){var r=parseInt(t.str,10);if(isNaN(r)){var i="edit distance must be numeric";throw new n.QueryParseError(i,t.start,t.end)}e.currentClause.editDistance=r;var s=e.peekLexeme();if(null==s)return void e.nextClause();switch(s.type){case n.QueryLexer.TERM:return e.nextClause(),n.QueryParser.parseTerm;case n.QueryLexer.FIELD:return e.nextClause(),n.QueryParser.parseField;case n.QueryLexer.EDIT_DISTANCE:return n.QueryParser.parseEditDistance;case n.QueryLexer.BOOST:return n.QueryParser.parseBoost;case n.QueryLexer.PRESENCE:return e.nextClause(),n.QueryParser.parsePresence;default:throw i="Unexpected lexeme type '"+s.type+"'",new n.QueryParseError(i,s.start,s.end)}}},n.QueryParser.parseBoost=function(e){var t=e.consumeLexeme();if(null!=t){var r=parseInt(t.str,10);if(isNaN(r)){var i="boost must be numeric";throw new n.QueryParseError(i,t.start,t.end)}e.currentClause.boost=r;var s=e.peekLexeme();if(null==s)return void e.nextClause();switch(s.type){case n.QueryLexer.TERM:return e.nextClause(),n.QueryParser.parseTerm;case n.QueryLexer.FIELD:return e.nextClause(),n.QueryParser.parseField;case n.QueryLexer.EDIT_DISTANCE:return n.QueryParser.parseEditDistance;case n.QueryLexer.BOOST:return n.QueryParser.parseBoost;case n.QueryLexer.PRESENCE:return e.nextClause(),n.QueryParser.parsePresence;default:throw i="Unexpected lexeme type '"+s.type+"'",new n.QueryParseError(i,s.start,s.end)}}},function(r,n){"function"==typeof define&&define.amd?define(n):"object"==typeof e?t.exports=n():r.lunr=n()}(this,(function(){return n}))}()},()=>(t||e((t={exports:{}}).exports,t),t.exports)),u=[];function c(e,t){u.push({selector:t,constructor:e})}var d=((e,t,l)=>(l=null!=e?r(o(e)):{},((e,t,r,o)=>{if(t&&"object"==typeof t||"function"==typeof t)for(let r of s(t))!a.call(e,r)&&undefined!==r&&n(e,r,{get:()=>t[r],enumerable:!(o=i(t,r))||o.enumerable});return e})(e&&e.__esModule?l:n(l,"default",{value:e,enumerable:!0}),e)))(l());function h(e,t){let r=e.querySelector(".current");if(r){let e=r;if(1===t)do{e=e.nextElementSibling??void 0}while(e instanceof HTMLElement&&null==e.offsetParent);else do{e=e.previousElementSibling??void 0}while(e instanceof HTMLElement&&null==e.offsetParent);e&&(r.classList.remove("current"),e.classList.add("current"))}else r=e.querySelector(1==t?"li:first-child":"li:last-child"),r&&r.classList.add("current")}function f(e,t){if(""===t)return e;let r=e.toLocaleLowerCase(),n=t.toLocaleLowerCase(),i=[],s=0,o=r.indexOf(n);for(;-1!=o;)i.push(m(e.substring(s,o)),`<b>${m(e.substring(o,o+n.length))}</b>`),s=o+n.length,o=r.indexOf(n,s);return i.push(m(e.substring(s))),i.join("")}var p={"&":"&amp;","<":"&lt;",">":"&gt;","'":"&#039;",'"':"&quot;"};function m(e){return e.replace(/[&<>"'"]/g,(e=>p[e]))}var y,g=class{constructor(e){this.el=e.el,this.app=e.app}},v="mousedown",x="mousemove",w="mouseup",E={x:0,y:0},L=!1,b=!1,k=!1,S=/Android|webOS|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i.test(navigator.userAgent);document.documentElement.classList.add(S?"is-mobile":"not-mobile"),S&&"ontouchstart"in document.documentElement&&(v="touchstart",x="touchmove",w="touchend"),document.addEventListener(v,(e=>{b=!0,k=!1;let t="touchstart"==v?e.targetTouches[0]:e;E.y=t.pageY||0,E.x=t.pageX||0})),document.addEventListener(x,(e=>{if(b&&!k){let t="touchstart"==v?e.targetTouches[0]:e,r=E.x-(t.pageX||0),n=E.y-(t.pageY||0);k=Math.sqrt(r*r+n*n)>10}})),document.addEventListener(w,(()=>{b=!1})),document.addEventListener("click",(e=>{L&&(e.preventDefault(),e.stopImmediatePropagation(),L=!1)}));try{y=localStorage}catch{y={getItem:()=>null,setItem(){}}}var Q=y,P=document.head.appendChild(document.createElement("style"));function T(e){document.documentElement.dataset.theme=e}P.dataset.for="filters",function(){let e=document.getElementById("tsd-search");if(!e)return;let t=document.getElementById("tsd-search-script");e.classList.add("loading"),t&&(t.addEventListener("error",(()=>{e.classList.remove("loading"),e.classList.add("failure")})),t.addEventListener("load",(()=>{e.classList.remove("loading"),e.classList.add("ready")})),window.searchData&&e.classList.remove("loading"));let r=document.querySelector("#tsd-search input"),n=document.querySelector("#tsd-search .results");if(!r||!n)throw new Error("The input field or the result list wrapper was not found");let i=!1;n.addEventListener("mousedown",(()=>i=!0)),n.addEventListener("mouseup",(()=>{i=!1,e.classList.remove("has-focus")})),r.addEventListener("focus",(()=>e.classList.add("has-focus"))),r.addEventListener("blur",(()=>{i||(i=!1,e.classList.remove("has-focus"))}));let s={base:e.dataset.base+"/"};!function(e,t,r,n){r.addEventListener("input",((e,t=100)=>{let r;return()=>{clearTimeout(r),r=setTimeout((()=>e()),t)}})((()=>{!function(e,t,r,n){if(function(e,t){e.index||window.searchData&&(t.classList.remove("loading"),t.classList.add("ready"),e.data=window.searchData,e.index=d.Index.load(window.searchData.index))}(n,e),!n.index||!n.data)return;t.textContent="";let i=r.value.trim(),s=i?n.index.search(`*${i}*`):[];for(let e=0;e<s.length;e++){let t=s[e],r=n.data.rows[Number(t.ref)],o=1;r.name.toLowerCase().startsWith(i.toLowerCase())&&(o*=1+1/(1+Math.abs(r.name.length-i.length))),t.score*=o}s.sort(((e,t)=>t.score-e.score));for(let e=0,r=Math.min(10,s.length);e<r;e++){let r=n.data.rows[Number(s[e].ref)],o=f(r.name,i);globalThis.DEBUG_SEARCH_WEIGHTS&&(o+=` (score: ${s[e].score.toFixed(2)})`),r.parent&&(o=`<span class="parent">${f(r.parent,i)}.</span>${o}`);let a=document.createElement("li");a.classList.value=r.classes??"";let l=document.createElement("a");l.href=n.base+r.url,l.innerHTML=o,a.append(l),t.appendChild(a)}}(e,t,r,n)}),200));let i=!1;r.addEventListener("keydown",(e=>{i=!0,"Enter"==e.key?function(e,t){let r=e.querySelector(".current");if(r||(r=e.querySelector("li:first-child")),r){let e=r.querySelector("a");e&&(window.location.href=e.href),t.blur()}}(t,r):"Escape"==e.key?r.blur():"ArrowUp"==e.key?h(t,-1):"ArrowDown"===e.key?h(t,1):i=!1})),r.addEventListener("keypress",(e=>{i&&e.preventDefault()})),document.body.addEventListener("keydown",(e=>{e.altKey||e.ctrlKey||e.metaKey||!r.matches(":focus")&&"/"===e.key&&(r.focus(),e.preventDefault())}))}(e,n,r,s)}(),c(class extends g{constructor(e){super(e),this.className=this.el.dataset.toggle||"",this.el.addEventListener(w,(e=>this.onPointerUp(e))),this.el.addEventListener("click",(e=>e.preventDefault())),document.addEventListener(v,(e=>this.onDocumentPointerDown(e))),document.addEventListener(w,(e=>this.onDocumentPointerUp(e)))}setActive(e){if(this.active==e)return;this.active=e,document.documentElement.classList.toggle("has-"+this.className,e),this.el.classList.toggle("active",e);let t=(this.active?"to-has-":"from-has-")+this.className;document.documentElement.classList.add(t),setTimeout((()=>document.documentElement.classList.remove(t)),500)}onPointerUp(e){k||(this.setActive(!0),e.preventDefault())}onDocumentPointerDown(e){if(this.active){if(e.target.closest(".col-sidebar, .tsd-filter-group"))return;this.setActive(!1)}}onDocumentPointerUp(e){if(!k&&this.active&&e.target.closest(".col-sidebar")){let t=e.target.closest("a");if(t){let e=window.location.href;-1!=e.indexOf("#")&&(e=e.substring(0,e.indexOf("#"))),t.href.substring(0,e.length)==e&&setTimeout((()=>this.setActive(!1)),250)}}}},"a[data-toggle]"),c(class extends g{constructor(e){super(e),this.summary=this.el.querySelector(".tsd-accordion-summary"),this.icon=this.summary.querySelector("svg"),this.key=`tsd-accordion-${this.summary.dataset.key??this.summary.textContent.trim().replace(/\s+/g,"-").toLowerCase()}`;let t=Q.getItem(this.key);this.el.open=t?"true"===t:this.el.open,this.el.addEventListener("toggle",(()=>this.update())),this.update()}update(){this.icon.style.transform=`rotate(${this.el.open?0:-90}deg)`,Q.setItem(this.key,this.el.open.toString())}},".tsd-index-accordion"),c(class extends g{constructor(e){super(e),this.key=`filter-${this.el.name}`,this.value=this.el.checked,this.el.addEventListener("change",(()=>{this.setLocalStorage(this.el.checked)})),this.setLocalStorage(this.fromLocalStorage()),P.innerHTML+=`html:not(.${this.key}) .tsd-is-${this.el.name} { display: none; }\n`}fromLocalStorage(){let e=Q.getItem(this.key);return e?"true"===e:this.el.checked}setLocalStorage(e){Q.setItem(this.key,e.toString()),this.value=e,this.handleValueChange()}handleValueChange(){this.el.checked=this.value,document.documentElement.classList.toggle(this.key,this.value),this.app.filterChanged(),document.querySelectorAll(".tsd-index-section").forEach((e=>{e.style.display="block";let t=Array.from(e.querySelectorAll(".tsd-index-link")).every((e=>null==e.offsetParent));e.style.display=t?"none":"block"}))}},".tsd-filter-item input[type=checkbox]");var O=document.getElementById("tsd-theme");O&&function(e){let t=Q.getItem("tsd-theme")||"os";e.value=t,T(t),e.addEventListener("change",(()=>{Q.setItem("tsd-theme",e.value),T(e.value)}))}(O);var I=new class{constructor(){this.alwaysVisibleMember=null,this.createComponents(document.body),this.ensureActivePageVisible(),this.ensureFocusedElementVisible(),this.listenForCodeCopies(),window.addEventListener("hashchange",(()=>this.ensureFocusedElementVisible()))}createComponents(e){u.forEach((t=>{e.querySelectorAll(t.selector).forEach((e=>{e.dataset.hasInstance||(new t.constructor({el:e,app:this}),e.dataset.hasInstance=String(!0))}))}))}filterChanged(){this.ensureFocusedElementVisible()}ensureActivePageVisible(){let e=document.querySelector(".tsd-navigation .current"),t=e?.parentElement;for(;t&&!t.classList.contains(".tsd-navigation");)t instanceof HTMLDetailsElement&&(t.open=!0),t=t.parentElement;if(e){let t=e.getBoundingClientRect().top-document.documentElement.clientHeight/4;document.querySelector(".site-menu").scrollTop=t}}ensureFocusedElementVisible(){if(this.alwaysVisibleMember&&(this.alwaysVisibleMember.classList.remove("always-visible"),this.alwaysVisibleMember.firstElementChild.remove(),this.alwaysVisibleMember=null),!location.hash)return;let e=document.getElementById(location.hash.substring(1));if(!e)return;let t=e.parentElement;for(;t&&"SECTION"!==t.tagName;)t=t.parentElement;if(t&&null==t.offsetParent){this.alwaysVisibleMember=t,t.classList.add("always-visible");let e=document.createElement("p");e.classList.add("warning"),e.textContent="This member is normally hidden due to your filter settings.",t.prepend(e)}}listenForCodeCopies(){document.querySelectorAll("pre > button").forEach((e=>{let t;e.addEventListener("click",(()=>{e.previousElementSibling instanceof HTMLElement&&navigator.clipboard.writeText(e.previousElementSibling.innerText.trim()),e.textContent="Copied!",e.classList.add("visible"),clearTimeout(t),t=setTimeout((()=>{e.classList.remove("visible"),t=setTimeout((()=>{e.textContent="Copy"}),100)}),1e3)}))}))}};Object.defineProperty(window,"app",{value:I}),document.querySelectorAll("summary a").forEach((e=>{e.addEventListener("click",(()=>{location.assign(e.href)}))}))})();