import { useState } from "react";
import useApi from "../../../../hooks/useApi";
import { Readability } from "@mozilla/readability";
import { mdiImport } from "@mdi/js";
import { Icon } from "@mdi/react";
import Spinner from "../../../../components/Spinner/Spinner";
import styles from "./styles.module.css";
import { fetch } from "@tauri-apps/plugin-http";
import IncrementalReading from "../../../../api/cells/valueObjects/incrementalReading";

interface Props {
	onImport: (newValue: IncrementalReading) => void;
}

export default function ImportContainer({ onImport }: Props) {
	const [url, setUrl] = useState("");
	const { callApi, errorMessage, clearErrorMessage, isSendingRequest } =
		useApi();

	const handleSubmit = (e: React.SubmitEvent) => {
		e.preventDefault();
		clearErrorMessage();

		if (isSendingRequest) return;

		void callApi(async () => {
			const response = await fetch(url);
			const html = await response.text();
			if (!response.status.toString().startsWith("2")) {
				throw new Error("Error: " + html);
			}
			const parser = new DOMParser();
			const doc = parser.parseFromString(html, "text/html");
			// Readability needs the URL to resolve relative links
			// Set the base URL via a <base> tag
			const base = doc.createElement("base");
			base.href = url;
			doc.head.prepend(base);

			const article = new Readability(doc).parse();

			onImport({
				title: article?.title ?? null,
				content: article?.content ?? null,
				priority: "normal",
				completed: false,
				source: {
					type: "url",
					url,
				},
			});
		});
	};

	return (
		<form className={styles.verticalForm} onSubmit={handleSubmit}>
			<input
				type="text"
				placeholder="Enter URL to import"
				value={url}
				onChange={e => setUrl(e.target.value)}
				required
			/>

			{errorMessage && (
				<p className={styles.errorMessage}>{errorMessage}</p>
			)}

			<button
				className={`primary ${styles.rowButton}`}
				disabled={isSendingRequest}>
				{!isSendingRequest && (
					<>
						<Icon path={mdiImport} size={1} />
						<span>Import</span>
					</>
				)}
				{isSendingRequest && <Spinner size={0.5} />}
			</button>
		</form>
	);
}
