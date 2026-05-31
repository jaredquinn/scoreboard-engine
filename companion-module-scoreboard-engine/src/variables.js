module.exports = {
	GetVariableDefinitions(widgets) {
		const defs = [];
		for (const id of Object.keys(widgets)) {
			defs.push({ variableId: id, name: id.replace(/_/g, ' ') });
			if (widgets[id].type === 'Switch') {
				defs.push({ variableId: `${id}_display`, name: `${id} Display Value` });
			}
			if (widgets[id].type === 'List') {
				defs.push({ variableId: `${id}_index`, name: `${id} Raw Index Value` });
			}
			if (widgets[id].type === 'Team') {
				defs.push({ variableId: `${id}_name`, name: `${id} Full Name` });
				defs.push({ variableId: `${id}_primary_color`, name: `${id} Primary Color` });
				defs.push({ variableId: `${id}_secondary_color`, name: `${id} Secondary Color` });
			}
			if (widgets[id].type === 'Timer') {
				defs.push({ variableId: `${id}_running`, name: `${id} Status Boolean` });
				defs.push({ variableId: `${id}_running_state`, name: `${id} Status Text` });
				defs.push({ variableId: `${id}_raw`, name: `${id} Raw Value` });
				defs.push({ variableId: `${id}_paused_formatted`, name: `${id} Formatted Stoppage Time` });
				defs.push({ variableId: `${id}_paused_time`, name: `${id} Raw Stoppage Time` });
				defs.push({ variableId: `${id}_total_formatted`, name: `${id} Formatted Total Time` });
				defs.push({ variableId: `${id}_total_time`, name: `${id} Raw Total Time` });
				defs.push({ variableId: `${id}_paused`, name: `${id} Currently Paused` });
			}
		}
		return defs;
	},

	GetVariableValues(widgets, ss) {
		const values = {};

		for (const [id, w] of Object.entries(widgets)) {
			const { type, data } = w;

			if (type === 'Switch') {
				values[id] = data.value;
				values[`${id}_display`] = data.value ? data.display_true : data.display_false;
			} else if (type === 'Counter') {
				values[id] = data.value;
			} else if (type == 'Team') {
				values[`${id}_name`] = data.name;
				values[`${id}_primary_color`] = data.primary_color;
				values[`${id}_secondary_color`] = data.secondary_color;
				values[id] = data.short_name;
			} else if (type === 'Timer') {
				values[`${id}_paused_formatted`] = data.paused_formatted;
				values[`${id}_paused_time`] = data.paused_time;
				values[`${id}_total_formatted`] = data.total_formatted;
				values[`${id}_total_time`] = data.total_time;
				values[`${id}_paused`] = data.paused;
				values[`${id}_running`] = data.running;
				values[`${id}_running_state`] = data.running ? 'RUN' : 'STOP';
				values[`${id}_raw`] = data.seconds;
				values[id] = data.formatted_time;
			} else if (type === 'List') {
				const { index, options } = data;
				values[id] = options[index] || '---';
				values[`${id}_index`] = index;
			} else if (type === 'Text') {
				values[id] = data.content;
			} else if (type === 'Calculation') {
				values[id] = data.value;
			}
		}
		return values;
	},
}
