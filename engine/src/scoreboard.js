
window.addEventListener('DOMContentLoaded', () => {
        console.log('𖣘  Scoreboard Engine Initializing');
	let values = {};
        const connect = () => {
		const es = new EventSource('/sse');

		es.onopen = () => {
			    console.log("🔌 Connected to Scoreboard Engine");
		};

		es.onmessage = (e) => {
			    const data = JSON.parse(e.data);
			    const widgets = data[0];
			    
			    Object.entries(widgets).forEach(([id, w]) => {
				    if(!id in values || values[id] != w) {
			    		values[id] = w;
				        console.log('updated ' + id + ' to ' + w);
					var els = document.querySelectorAll('[data-foreground="' + id + '"]');
					els.forEach(el => {
						el.style.color = w;
					});
					var els = document.querySelectorAll('[data-background="' + id + '"]');
					els.forEach(el => {
						el.style.backgroundColor = w;
					});
					var els = document.querySelectorAll('[data-bind="' + id + '"]');
					els.forEach(el => {
						el.innerHTML = w;
					});
					var els = document.querySelectorAll('[data-visible="' + id + '"]');
					els.forEach(el => {
						if(w == true) {
							el.style.visibility = 'visible';
						} else {
							el.style.visibility = 'hidden';
						}
					});

			    	    }
			    });
		};

		es.onerror = () => {
			    console.error("🔌 Connection lost. Retrying...");
			    es.close();
			    setTimeout(connect, 2000); // Auto-reconnect after 2s
		};
        };
        connect();
});

