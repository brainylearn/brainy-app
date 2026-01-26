export type StreamLlmResponseEvent =
	| {
			event: "inProgress";
			data: string;
	  }
	| {
			event: "finished";
	  }
	| {
			event: "error";
			data: string;
	  };
