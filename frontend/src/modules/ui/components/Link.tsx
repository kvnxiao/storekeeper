import { createLink } from "@tanstack/react-router";
import { Link as RACLink } from "react-aria-components";

/**
 * Link component that integrates React Aria's Link with TanStack Router.
 * Use this instead of TanStack Router's Link for proper client-side navigation.
 */
export const Link = createLink(RACLink);
