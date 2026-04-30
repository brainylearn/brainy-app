import { useState, useCallback } from "react";
import errorToString from "../utils/errorToString";

export type CallApiFn = <T>(
	fetch: () => Promise<T>,
	onFinally?: () => Promise<T>,
) => Promise<T | undefined>;

export default function useApi() {
	const [isSendingRequest, setIsSendingRequest] = useState<boolean>(false);
	const [errorMessage, setErrorMessage] = useState<string | null>(null);

	const callApi: CallApiFn = useCallback(async (fetch, onFinally) => {
		setIsSendingRequest(true);

		try {
			return await fetch();
		} catch (e) {
			console.error(e);
			setErrorMessage(errorToString(e));
		} finally {
			setIsSendingRequest(false);
			if (onFinally) await onFinally();
		}
	}, []);

	const clearErrorMessage = useCallback(() => setErrorMessage(null), []);

	return {
		isSendingRequest,
		errorMessage,
		callApi,
		clearErrorMessage,
	};
}
