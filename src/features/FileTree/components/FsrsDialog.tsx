import styles from "./styles.module.css";
import Dialog from "../../../components/Dialog/Dialog";
import Form, {
	FormButtons,
	FormHeader,
	FormRows,
} from "../../../components/Form/Form";
import { mdiPlusBoxOutline, mdiTuneVariant } from "@mdi/js";
import Icon from "@mdi/react";
import { useCallback, useEffect, useState } from "react";
import errorToString from "../../../utils/errorToString";
import Alert from "../../../components/Alert/Alert";
import FsrsProfile from "../../../types/backend/entity/fsrsProfile";
import {
	getAllFsrsProfiles,
	getFileFsrsProfile,
	getFolderFsrsProfile,
} from "../../../api/fsrsApi";
import { ROOT_FOLDER_ID } from "../../../config/constants";

interface Props {
	id: string;
	isFolder: boolean;
	onClose: () => void;
}

export default function FsrsDialog({ id, isFolder, onClose }: Props) {
	const [allFsrsProfiles, setAllFsrsProfiles] = useState<FsrsProfile[]>([]);
	const [errorMessage, setErrorMessage] = useState("");
	const [chosenProfile, setChosenProfile] = useState("");
	const [profileState, setProfileState] = useState<FsrsProfile | null>();

	const isRoot = id === ROOT_FOLDER_ID;

	const handleSubmit = (e: React.FormEvent) => {
		e.preventDefault();
		onClose();
	};

	const executeRequest = useCallback(
		async (cb: () => Promise<void>): Promise<void> => {
			try {
				await cb();
			} catch (e) {
				console.error(e);
				setErrorMessage(errorToString(e));
			}
		},
		[],
	);

	useEffect(() => {
		void (async () => {
			setAllFsrsProfiles(await getAllFsrsProfiles());
			const itemProfile = isFolder
				? await getFolderFsrsProfile(id)
				: await getFileFsrsProfile(id);

			// TODO: change return type
			setChosenProfile(
				itemProfile === "inherit" ? "inherit" : itemProfile.id,
			);
			// TODO: get profile endpoint
		})();
	}, [executeRequest, isFolder, id]);

	return (
		<Dialog onHide={onClose} focusTrap className={styles.fsrsDialog}>
			<Form onSubmit={handleSubmit}>
				<FormHeader icon={mdiTuneVariant} title="FSRS Profile" />

				{profileState && (
					<FormRows
						rows={[
							{
								label: "Profile",
								labelHtmlFor: "profile",
								children: (
									<div className={styles.chooseProfileRow}>
										<select
											id="profile"
											value={chosenProfile}
											onChange={e =>
												setChosenProfile(e.target.value)
											}
											autoFocus>
											{!isRoot && (
												<option value="inherit">
													Inherit from parent
												</option>
											)}
											{allFsrsProfiles.map(profile => (
												<option
													value={profile.id}
													key={profile.id}>
													{profile.name}
												</option>
											))}
										</select>
										<button
											className="transparent"
											type="button"
											title="Clone profile">
											<Icon
												path={mdiPlusBoxOutline}
												size={1}
											/>
										</button>
									</div>
								),
							},
							{
								label: "Name",
								labelHtmlFor: "name",
								children: (
									<input
										id="name"
										type="text"
										minLength={1}
										value={profileState.name}
										onChange={e =>
											setProfileState({
												...profileState,
												name: e.target.value,
											})
										}
										readOnly={chosenProfile === "inherit"}
										required
									/>
								),
							},
							{
								label: "Request retention",
								labelHtmlFor: "request-retention",
								children: (
									<input
										id="request-retention"
										type="number"
										readOnly={chosenProfile === "inherit"}
										value={profileState.requestRetention}
										onChange={e =>
											setProfileState({
												...profileState,
												requestRetention: Number(
													e.target.value,
												),
											})
										}
										required
									/>
								),
							},
							{
								label: "Maximum interval",
								labelHtmlFor: "maximum-interval",
								children: (
									<input
										id="maximum-interval"
										type="number"
										readOnly={chosenProfile === "inherit"}
										value={profileState.maximumInterval}
										onChange={e =>
											setProfileState({
												...profileState,
												maximumInterval: Number(
													e.target.value,
												),
											})
										}
										required
									/>
								),
							},
							{
								label: "Weights",
								labelHtmlFor: "weights",
								children: (
									<textarea
										id="weights"
										placeholder="TODO:"
										rows={3}
										readOnly={chosenProfile === "inherit"}
										value={profileState.maximumInterval}
										// TODO: better validation
										onChange={e =>
											setProfileState({
												...profileState,
												weights: e.target.value
													.split(" ")
													.map(w => Number(w)),
											})
										}
										required
									/>
								),
							},
						]}
					/>
				)}

				{errorMessage && (
					<Alert type="error" onClose={() => setErrorMessage("")}>
						<p>{errorMessage}</p>
					</Alert>
				)}

				<FormButtons onClose={onClose} submitText="Save" />
			</Form>
		</Dialog>
	);
}
