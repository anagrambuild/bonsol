import { useState } from 'react';
import {
  Box,
  Badge,
  Button,
  Flex,
  Stack,
  Text,
  Spinner,
  Select,
  Table,
  Thead,
  Tbody,
  Tr,
  Th,
  Td,
} from '@chakra-ui/react';
import { Link } from 'react-router-dom';
import { useQuery } from 'react-query';
import { getRequests } from '../services/api';
import { RequestStatus, ChainStatus, RequestFilters } from '../types';

const RequestList = () => {
  const [filters, setFilters] = useState<RequestFilters>({
    page: 1,
    limit: 10,
  });

  const { data, isLoading, error } = useQuery(
    ['requests', filters],
    () => getRequests(filters),
    {
      keepPreviousData: true,
    }
  );

  const getStatusBadge = (status: RequestStatus) => {
    const colors = {
      [RequestStatus.PENDING]: 'yellow',
      [RequestStatus.IN_PROGRESS]: 'blue',
      [RequestStatus.COMPLETED]: 'green',
      [RequestStatus.FAILED]: 'red',
    };
    return <Badge colorScheme={colors[status]}>{status}</Badge>;
  };

  const getChainStatusBadge = (status: ChainStatus) => {
    const colors = {
      [ChainStatus.UNCLAIMED]: 'gray',
      [ChainStatus.CLAIMED]: 'purple',
      [ChainStatus.EXECUTED]: 'teal',
      [ChainStatus.POSTED]: 'green',
    };
    return <Badge colorScheme={colors[status]}>{status}</Badge>;
  };

  if (error) {
    return <Text color="red.500">Error loading requests</Text>;
  }

  return (
    <Box>
      <Flex justify="space-between" mb={6}>
        <Stack direction="row" spacing={4}>
          <Select
            value={filters.status}
            onChange={(e) => setFilters({ ...filters, status: e.target.value as RequestStatus })}
            placeholder="Filter by Status"
            w="200px"
          >
            {Object.values(RequestStatus).map((status) => (
              <option key={status} value={status}>
                {status}
              </option>
            ))}
          </Select>
          <Select
            value={filters.chainStatus}
            onChange={(e) => setFilters({ ...filters, chainStatus: e.target.value as ChainStatus })}
            placeholder="Filter by Chain Status"
            w="200px"
          >
            {Object.values(ChainStatus).map((status) => (
              <option key={status} value={status}>
                {status}
              </option>
            ))}
          </Select>
        </Stack>
      </Flex>

      {isLoading ? (
        <Flex justify="center" p={8}>
          <Spinner size="lg" />
        </Flex>
      ) : (
        <>
          <Box overflowX="auto">
            <Table variant="simple">
              <Thead>
                <Tr>
                  <Th>ID</Th>
                  <Th>Status</Th>
                  <Th>Chain Status</Th>
                  <Th>Image ID</Th>
                  <Th>Created At</Th>
                  <Th>Action</Th>
                </Tr>
              </Thead>
              <Tbody>
                {data?.data.map((request) => (
                  <Tr key={request.id}>
                    <Td>{request.id}</Td>
                    <Td>{getStatusBadge(request.status)}</Td>
                    <Td>{getChainStatusBadge(request.chainStatus)}</Td>
                    <Td>{request.imageId?.slice(0, 8)}...</Td>
                    <Td>{new Date(request.createdAt).toLocaleString()}</Td>
                    <Td>
                      <Button
                        as={Link}
                        to={`/request/${request.id}`}
                        size="sm"
                        colorScheme="blue"
                      >
                        View Details
                      </Button>
                    </Td>
                  </Tr>
                ))}
              </Tbody>
            </Table>
          </Box>

          <Flex justify="space-between" align="center" mt={4}>
            <Button
              onClick={() => setFilters({ ...filters, page: filters.page - 1 })}
              isDisabled={filters.page === 1}
              size="sm"
            >
              Previous
            </Button>
            <Text>
              Page {filters.page} of {data?.meta.totalPages || 1}
            </Text>
            <Button
              onClick={() => setFilters({ ...filters, page: filters.page + 1 })}
              isDisabled={!data?.meta.totalPages || filters.page === data.meta.totalPages}
              size="sm"
            >
              Next
            </Button>
          </Flex>
        </>
      )}
    </Box>
  );
};

export default RequestList; 
