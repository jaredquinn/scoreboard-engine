window.addEventListener('DOMContentLoaded', () => {
    console.log('𖣘  Scoreboard Engine Initializing');
    let values = {};
    let elementCache = {};
    let reconnectDelay = 2000;
    let es = null;

    /**
     * Correctly loops through ALL potential scoreboard data attributes 
     * ensuring an element is cached for every single ID it cares about.
     */
    const indexElements = () => {
        elementCache = {};
        const elements = document.querySelectorAll('[data-foreground], [data-background], [data-bind], [data-visible], [data-invisible]');
        
        elements.forEach(el => {
            // Check all 5 possible attributes individually
            const attributes = ['foreground', 'background', 'bind', 'visible', 'invisible'];
            
            attributes.forEach(attr => {
                const id = el.dataset[attr];
                if (id) {
                    if (!elementCache[id]) {
                        elementCache[id] = [];
                    }
                    // Avoid duplicate entries for the same element under one ID
                    if (!elementCache[id].includes(el)) {
                        elementCache[id].push(el);
                    }
                }
            });
        });
    };

    const connect = () => {
        es = new EventSource('/sse');

        es.onopen = () => {
            console.log("🔌 Connected to Scoreboard Engine");
            reconnectDelay = 2000;
            document.dispatchEvent(new CustomEvent('scoreboard:connected'));
        };

        es.onmessage = (e) => {
            const data = JSON.parse(e.data);
            const widgets = data[0] || {};

            Object.entries(widgets).forEach(([id, w]) => {
                if (values[id] != w) {
                    values[id] = w;
                    console.log('updated ' + id + ' to ' + w);

                    document.dispatchEvent(new CustomEvent('scoreboard:update', {
                        detail: { id, value: w }
                    }));

                    // Pull elements matching this specific telemetry ID from our updated cache map
                    const els = elementCache[id] || [];
                    els.forEach(el => {
                        const allowed = el.dispatchEvent(new CustomEvent('scoreboard:element-update', {
                            detail: { id, value: w },
                            cancelable: true
                        }));

                        if (!allowed) return;

                        // Check every condition independently so one element can handle multiple jobs
                        if (el.dataset.foreground === id) {
                            el.style.color = w;
                        }
                        if (el.dataset.background === id) {
                            el.style.backgroundColor = w;
                        }
                        if (el.dataset.bind === id) {
                            el.innerHTML = w;
                        }
                        if (el.dataset.visible === id) {
                            el.style.visibility = (w == true) ? 'visible' : 'hidden';
                        }
                        if (el.dataset.invisible === id) {
                            el.style.visibility = (w == true) ? 'hidden' : 'visible';
                        }
                    });
                }
            });
        };

        es.onerror = () => {
            console.error("🔌 Connection lost. Retrying...");
            es.close();
            document.dispatchEvent(new CustomEvent('scoreboard:disconnected'));
            
            const jitter = Math.random() * 1000;
            setTimeout(() => {
                reconnectDelay = Math.min(reconnectDelay * 1.5, 15000);
                connect();
            }, reconnectDelay + jitter);
        };
    };

    indexElements();
    connect();
});
