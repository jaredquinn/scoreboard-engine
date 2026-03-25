const { InstanceBase, runEntrypoint, InstanceStatus } = require('@companion-module/base');
const EventSource = require('eventsource');
const { GetConfigFields } = require('./config');
const { GetActionDefinitions } = require('./actions');
const { GetFeedbackDefinitions } = require('./feedbacks');
const { GetVariableDefinitions, GetVariableValues } = require('./variables');

class ScoreboardModule extends InstanceBase {
	constructor(internal) {
		super(internal);
		this.widgets = {};
	}

	async init(config) {
		this.config = config;
		this.updateStatus(InstanceStatus.Connecting);

		await this.discoverWidgets();

		this.initSSE();
		this.updateActions();
		this.updateFeedbacks();
	}

	async destroy() {
		if (this.sse) {
			this.sse.close();
		}
		this.log('debug', 'Module destroyed');
	}

	async configUpdated(config) {
		this.config = config;
		await this.discoverWidgets();
		this.initSSE();
	}

	getConfigFields() {
		return GetConfigFields();
	}

	async discoverWidgets() {
		if (!this.config.host) return;

		try {
			const response = await fetch(`http://${this.config.host}:${this.config.port}/widgets`);
			this.widgets = await response.json();
			
			this.setVariableDefinitions(GetVariableDefinitions(this.widgets));
			this.setVariableValues(GetVariableValues(this.widgets));
			
			this.updateStatus(InstanceStatus.Ok);
		} catch (e) {
			this.updateStatus(InstanceStatus.ConnectionFailure, `API Unreachable: ${e.message}`);
		}
	}

	initSSE() {
		if (this.sse) this.sse.close();

		const url = `http://${this.config.host}:${this.config.port}/events`;
		this.sse = new EventSource(url);

		this.sse.onmessage = (event) => {
			try {
				const widgets = JSON.parse(event.data);
				this.widgets = widgets; // Update cache
				this.setVariableValues(GetVariableValues(widgets));
				this.checkFeedbacks();
			} catch (e) {
				this.log('error', `SSE Parse Error: ${e.message}`);
			}
		};

		this.sse.onerror = () => {
			this.updateStatus(InstanceStatus.Disconnected, 'SSE Lost');
		};
	}

	// Helper for Actions to talk back to Rust
	async sendUpdate(id, payload) {
		const url = `http://${this.config.host}:${this.config.port}/widgets/${id}/update`;
		try {
			await fetch(url, {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify(payload),
			});
		} catch (e) {
			this.log('error', `Update failed for ${id}: ${e.message}`);
		}
	}

	updateActions() {
		this.setActionDefinitions(GetActionDefinitions(this));
	}

	updateFeedbacks() {
		this.setFeedbackDefinitions(GetFeedbackDefinitions(this));
	}
}

runEntrypoint(ScoreboardModule, []);

